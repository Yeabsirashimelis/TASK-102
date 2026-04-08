use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::teams;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = teams)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = teams)]
pub struct NewTeam {
    pub name: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = teams)]
pub struct UpdateTeamChangeset {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub department: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Team> for TeamResponse {
    fn from(t: Team) -> Self {
        Self {
            id: t.id,
            name: t.name,
            description: t.description,
            department: t.department,
            location: t.location,
            is_active: t.is_active,
            created_by: t.created_by,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Serialize)]
pub struct TeamDetailResponse {
    #[serde(flatten)]
    pub team: TeamResponse,
    pub members: Vec<super::team_member::TeamMemberResponse>,
}

#[derive(Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 256))]
    pub name: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateTeamRequest {
    #[validate(length(min = 1, max = 256))]
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub department: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub is_active: Option<bool>,
}
