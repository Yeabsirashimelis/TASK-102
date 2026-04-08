use crate::models::order::OrderStatus;

/// Returns true if the state transition from `from` to `to` is valid.
pub fn valid_transition(from: &OrderStatus, to: &OrderStatus) -> bool {
    matches!(
        (from, to),
        (OrderStatus::Draft, OrderStatus::Open)
            | (OrderStatus::Open, OrderStatus::Tendering)
            | (OrderStatus::Tendering, OrderStatus::Paid)
            | (OrderStatus::Paid, OrderStatus::Closed)
            // Returns
            | (OrderStatus::Paid, OrderStatus::ReturnInitiated)
            | (OrderStatus::Closed, OrderStatus::ReturnInitiated)
            | (OrderStatus::ReturnInitiated, OrderStatus::Returned)
            // Reversals
            | (OrderStatus::Paid, OrderStatus::ReversalPending)
            | (OrderStatus::Closed, OrderStatus::ReversalPending)
            | (OrderStatus::ReversalPending, OrderStatus::Reversed)
    )
}

/// Returns the permission code required for a given target status transition.
/// Returns None if the base `order.transition` permission is sufficient.
pub fn extra_permission_for_transition(to: &OrderStatus) -> Option<&'static str> {
    match to {
        OrderStatus::ReturnInitiated => Some("order.return"),
        OrderStatus::ReversalPending => Some("order.reverse"),
        OrderStatus::Reversed => Some("order.reverse"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        assert!(valid_transition(&OrderStatus::Draft, &OrderStatus::Open));
        assert!(valid_transition(&OrderStatus::Open, &OrderStatus::Tendering));
        assert!(valid_transition(&OrderStatus::Tendering, &OrderStatus::Paid));
        assert!(valid_transition(&OrderStatus::Paid, &OrderStatus::Closed));
    }

    #[test]
    fn test_return_path() {
        assert!(valid_transition(&OrderStatus::Paid, &OrderStatus::ReturnInitiated));
        assert!(valid_transition(&OrderStatus::Closed, &OrderStatus::ReturnInitiated));
        assert!(valid_transition(&OrderStatus::ReturnInitiated, &OrderStatus::Returned));
    }

    #[test]
    fn test_reversal_path() {
        assert!(valid_transition(&OrderStatus::Paid, &OrderStatus::ReversalPending));
        assert!(valid_transition(&OrderStatus::Closed, &OrderStatus::ReversalPending));
        assert!(valid_transition(&OrderStatus::ReversalPending, &OrderStatus::Reversed));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!valid_transition(&OrderStatus::Draft, &OrderStatus::Paid));
        assert!(!valid_transition(&OrderStatus::Closed, &OrderStatus::Open));
        assert!(!valid_transition(&OrderStatus::Reversed, &OrderStatus::Draft));
        assert!(!valid_transition(&OrderStatus::Returned, &OrderStatus::Paid));
    }

    #[test]
    fn test_no_self_transitions() {
        assert!(!valid_transition(&OrderStatus::Draft, &OrderStatus::Draft));
        assert!(!valid_transition(&OrderStatus::Open, &OrderStatus::Open));
        assert!(!valid_transition(&OrderStatus::Paid, &OrderStatus::Paid));
    }

    #[test]
    fn test_cannot_skip_states() {
        assert!(!valid_transition(&OrderStatus::Draft, &OrderStatus::Tendering));
        assert!(!valid_transition(&OrderStatus::Draft, &OrderStatus::Closed));
        assert!(!valid_transition(&OrderStatus::Open, &OrderStatus::Paid));
        assert!(!valid_transition(&OrderStatus::Open, &OrderStatus::Closed));
    }

    #[test]
    fn test_terminal_states_have_no_exits() {
        let terminals = [OrderStatus::Reversed, OrderStatus::Returned];
        let all_states = [
            OrderStatus::Draft, OrderStatus::Open, OrderStatus::Tendering,
            OrderStatus::Paid, OrderStatus::Closed, OrderStatus::ReturnInitiated,
            OrderStatus::Returned, OrderStatus::ReversalPending, OrderStatus::Reversed,
        ];
        for terminal in &terminals {
            for target in &all_states {
                assert!(
                    !valid_transition(terminal, target),
                    "Terminal {:?} should not transition to {:?}", terminal, target
                );
            }
        }
    }

    #[test]
    fn test_extra_permission_return() {
        assert_eq!(
            extra_permission_for_transition(&OrderStatus::ReturnInitiated),
            Some("order.return")
        );
    }

    #[test]
    fn test_extra_permission_reversal() {
        assert_eq!(
            extra_permission_for_transition(&OrderStatus::ReversalPending),
            Some("order.reverse")
        );
        assert_eq!(
            extra_permission_for_transition(&OrderStatus::Reversed),
            Some("order.reverse")
        );
    }

    #[test]
    fn test_no_extra_permission_normal() {
        assert_eq!(extra_permission_for_transition(&OrderStatus::Open), None);
        assert_eq!(extra_permission_for_transition(&OrderStatus::Tendering), None);
        assert_eq!(extra_permission_for_transition(&OrderStatus::Paid), None);
        assert_eq!(extra_permission_for_transition(&OrderStatus::Closed), None);
    }
}
