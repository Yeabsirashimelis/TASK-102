use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::team_members;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = team_members)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub participant_id: Uuid,
    pub role_label: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Insertable)]
#[diesel(table_name = team_members)]
pub struct NewTeamMember {
    pub team_id: Uuid,
    pub participant_id: Uuid,
    pub role_label: Option<String>,
}

#[derive(Serialize)]
pub struct TeamMemberResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub participant_id: Uuid,
    pub role_label: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl From<TeamMember> for TeamMemberResponse {
    fn from(tm: TeamMember) -> Self {
        Self {
            id: tm.id,
            team_id: tm.team_id,
            participant_id: tm.participant_id,
            role_label: tm.role_label,
            joined_at: tm.joined_at,
            left_at: tm.left_at,
            is_active: tm.is_active,
        }
    }
}

#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub participant_id: Uuid,
    pub role_label: Option<String>,
}

#[derive(Deserialize)]
pub struct RemoveMemberRequest {
    pub participant_id: Uuid,
}
