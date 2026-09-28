use super::*;
use crate::tests::{Fixture, message_to};

fn queue(fx: &Fixture, bot: usize, body: &str, next_attempt_at: i64) -> Delivery {
    let (message, mut delivery) = message_to(&fx.crew.id, &fx.bots[bot].id, body);
    delivery.next_attempt_at = next_attempt_at;
    fx.store
        .insert_message(&message, &delivery, None)
        .expect("insert");
    delivery
}

fn due(fx: &Fixture, now: i64) -> Vec<DeliveryId> {
    let due = fx.store.due_deliveries(now).expect("due");
    due.into_iter().map(|d| d.id).collect()
}

#[test]
fn each_bot_sends_its_oldest_delivery_one_at_a_time() {
    let fx = Fixture::new();
    let a1 = queue(&fx, 0, "a1", 0);
    let a2 = queue(&fx, 0, "a2", 0);
    let b1 = queue(&fx, 1, "b1", 50);
    assert_eq!(due(&fx, 10), vec![a1.id.clone()], "b1 is not due yet");
    assert_eq!(due(&fx, 50), vec![a1.id.clone(), b1.id.clone()]);

    let sending = fx.store.claim_delivery(&a1.id, 50, 1_000).expect("claim");
    assert_eq!(sending.map(|d| d.state), Some(DeliveryState::Sending));
    assert_eq!(
        fx.store.claim_delivery(&a1.id, 50, 1_000).expect("again"),
        None
    );
    assert_eq!(due(&fx, 50), vec![b1.id.clone()], "a2 waits for a1");

    let sent = fx
        .store
        .finish_delivery(&a1.id, DeliveryOutcome::Sent, 60)
        .expect("finish")
        .expect("row");
    assert_eq!(sent.state, DeliveryState::Sent);
    assert_eq!(due(&fx, 60), [a2.id, b1.id]);
}

#[test]
fn a_failed_delivery_keeps_its_place_until_it_gives_up() {
    let fx = Fixture::new();
    let first = queue(&fx, 0, "first", 0);
    let second = queue(&fx, 0, "second", 0);
    fx.store.claim_delivery(&first.id, 0, 100).expect("claim");
    let failed = fx
        .store
        .finish_delivery(
            &first.id,
            DeliveryOutcome::Failed {
                error: "pipe gone",
                retry_at: Some(500),
            },
            10,
        )
        .expect("finish")
        .expect("row");
    assert_eq!((failed.state, failed.attempts), (DeliveryState::Pending, 1));
    assert_eq!(failed.last_error.as_deref(), Some("pipe gone"));
    assert!(due(&fx, 100).is_empty(), "the second waits behind it");
    assert_eq!(due(&fx, 500), vec![first.id.clone()]);

    let dead = fx
        .store
        .finish_delivery(
            &first.id,
            DeliveryOutcome::Failed {
                error: "pipe gone",
                retry_at: None,
            },
            600,
        )
        .expect("finish")
        .expect("row");
    assert_eq!((dead.state, dead.attempts), (DeliveryState::Dead, 2));
    assert_eq!(due(&fx, 600), [second.id]);
    assert_eq!(
        fx.store.delivery_backlog().expect("backlog"),
        DeliveryBacklog {
            pending: 1,
            dead: 1
        }
    );

    let retried = fx
        .store
        .retry_delivery(&first.id, 700)
        .expect("retry")
        .expect("row");
    assert_eq!(
        (retried.state, retried.attempts, retried.last_error),
        (DeliveryState::Pending, 0, None)
    );
    assert_eq!(
        fx.store.retry_delivery(&first.id, 700).expect("retry"),
        None
    );
}

#[test]
fn deferring_does_not_count_or_touch_updated_at() {
    let fx = Fixture::new();
    let delivery = queue(&fx, 0, "wait", 0);
    let deferred = fx
        .store
        .finish_delivery(&delivery.id, DeliveryOutcome::Defer { until: 5_000 }, 10)
        .expect("defer")
        .expect("row");
    assert_eq!(deferred.attempts, 0);
    assert_eq!(deferred.next_attempt_at, 5_000);
    assert_eq!(deferred.updated_at, delivery.updated_at);
}

#[test]
fn expired_leases_go_back_to_pending() {
    let fx = Fixture::new();
    let delivery = queue(&fx, 0, "lost", 0);
    fx.store
        .claim_delivery(&delivery.id, 0, 100)
        .expect("claim");
    assert!(fx.store.recover_leases(99).expect("early").is_empty());
    let recovered = fx.store.recover_leases(100).expect("recover");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].state, DeliveryState::Pending);
    assert_eq!(
        fx.store
            .deliveries(Some(DeliveryState::Pending), None)
            .expect("list")
            .len(),
        1
    );
}
