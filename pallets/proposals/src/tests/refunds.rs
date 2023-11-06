
#[test]
fn you_can_actually_refund_after_dispute_success() {
    build_test_externality().execute_with(|| {

    })
}

// The case where a project is in a dispute, and the dispute passes however, a milestone has also been approved
// before the refund has been called.
// Without the proper checks there will be a kind of double spend.
#[test]
fn refund_only_transfers_milestones_which_havent_been_withdrawn() {
    build_test_externality().execute_with(|| {

    })
}

#[test]
fn refund_check_refund_amount() {
    build_test_externality().execute_with(|| {

    })
}

#[test]
fn refund_takes_imbue_fee() {
    build_test_externality().execute_with(|| {

    })
}