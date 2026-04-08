use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::delegation::Delegation;
use crate::schema::delegations;

/// Returns all active delegations for a given user at the current time.
pub fn get_active_delegations(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<Delegation>, AppError> {
    let now = Utc::now();
    let results = delegations::table
        .filter(delegations::delegate_user_id.eq(user_id))
        .filter(delegations::is_active.eq(true))
        .filter(delegations::starts_at.le(now))
        .filter(delegations::ends_at.gt(now))
        .load::<Delegation>(conn)?;
    Ok(results)
}
