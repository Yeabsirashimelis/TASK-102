use actix_web::{web, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::team::*;
use crate::models::team_member::*;
use crate::rbac::guard::check_permission;
use crate::schema::{team_members, teams};

pub async fn create(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreateTeamRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "team.create", &mut conn)?;

    let new = NewTeam {
        name: body.name.clone(),
        description: body.description.clone(),
        department: body.department.clone(),
        location: body.location.clone(),
        created_by: auth.0.sub,
    };

    let team: Team = diesel::insert_into(teams::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(TeamResponse::from(team)))
}

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "team.read", &mut conn)?;

    let mut q = teams::table
        .filter(teams::is_active.eq(true))
        .into_boxed();

    match ctx.data_scope.as_str() {
        "department" => {
            if let Some(ref dept) = ctx.department {
                q = q.filter(teams::department.eq(dept));
            }
        }
        "location" => {
            if let Some(ref loc) = ctx.location {
                q = q.filter(teams::location.eq(loc));
            }
        }
        "individual" => {
            q = q.filter(teams::created_by.eq(ctx.user_id));
        }
        _ => {}
    }

    let results: Vec<Team> = q
        .select(Team::as_select())
        .order(teams::name.asc())
        .load(&mut conn)?;

    let responses: Vec<TeamResponse> = results.into_iter().map(TeamResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let team_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "team.read", &mut conn)?;

    let team: Team = teams::table
        .find(team_id)
        .select(Team::as_select())
        .first(&mut conn)?;

    ctx.enforce_scope(team.created_by, team.department.as_deref(), team.location.as_deref())?;

    let members: Vec<TeamMember> = team_members::table
        .filter(team_members::team_id.eq(team_id))
        .filter(team_members::is_active.eq(true))
        .select(TeamMember::as_select())
        .load(&mut conn)?;

    let response = TeamDetailResponse {
        team: TeamResponse::from(team),
        members: members.into_iter().map(TeamMemberResponse::from).collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn update(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTeamRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let team_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "team.update", &mut conn)?;

    let existing: Team = teams::table.find(team_id).select(Team::as_select()).first(&mut conn)?;
    ctx.enforce_scope(existing.created_by, existing.department.as_deref(), existing.location.as_deref())?;

    let changeset = UpdateTeamChangeset {
        name: body.name.clone(),
        description: body.description.clone(),
        department: body.department.clone(),
        location: body.location.clone(),
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let team: Team = diesel::update(teams::table.find(team_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(TeamResponse::from(team)))
}

pub async fn deactivate(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let team_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "team.delete", &mut conn)?;
    let t: Team = teams::table.find(team_id).select(Team::as_select()).first(&mut conn)?;
    ctx.enforce_scope(t.created_by, t.department.as_deref(), t.location.as_deref())?;

    diesel::update(teams::table.find(team_id))
        .set((
            teams::is_active.eq(false),
            teams::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// --- Member management ---

pub async fn add_member(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<AddMemberRequest>,
) -> Result<HttpResponse, AppError> {
    let team_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "team.manage_members", &mut conn)?;
    let t: Team = teams::table.find(team_id).select(Team::as_select()).first(&mut conn)?;
    ctx.enforce_scope(t.created_by, t.department.as_deref(), t.location.as_deref())?;

    let new = NewTeamMember {
        team_id,
        participant_id: body.participant_id,
        role_label: body.role_label.clone(),
    };

    let member: TeamMember = diesel::insert_into(team_members::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(TeamMemberResponse::from(member)))
}

pub async fn remove_member(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (team_id, participant_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "team.manage_members", &mut conn)?;
    let t: Team = teams::table.find(team_id).select(Team::as_select()).first(&mut conn)?;
    ctx.enforce_scope(t.created_by, t.department.as_deref(), t.location.as_deref())?;

    diesel::update(
        team_members::table
            .filter(team_members::team_id.eq(team_id))
            .filter(team_members::participant_id.eq(participant_id)),
    )
    .set((
        team_members::is_active.eq(false),
        team_members::left_at.eq(Some(Utc::now())),
    ))
    .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_members(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let team_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "team.read", &mut conn)?;
    let t: Team = teams::table.find(team_id).select(Team::as_select()).first(&mut conn)?;
    ctx.enforce_scope(t.created_by, t.department.as_deref(), t.location.as_deref())?;

    let members: Vec<TeamMember> = team_members::table
        .filter(team_members::team_id.eq(team_id))
        .filter(team_members::is_active.eq(true))
        .select(TeamMember::as_select())
        .load(&mut conn)?;

    let responses: Vec<TeamMemberResponse> =
        members.into_iter().map(TeamMemberResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}
