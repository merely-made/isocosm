use super::*;

#[test]
fn bridge_exposes_fixed_descriptor_and_closes_pump_with_stream() {
    let descriptor = EndpointDescriptor {
        label: "Isocosm family".into(),
        projections: Vec::new(),
    };
    let (bridge, pump) = BridgeEndpoint::new(
        AuthenticatedPeerKey([7; 32]),
        descriptor.clone(),
        EndpointBridgeLimits::default(),
    )
    .expect("bounded bridge");
    assert_eq!(bridge.describe(), descriptor);
    assert!(pump.try_next().unwrap().is_none());
    drop(bridge);
    assert!(pump.try_next().is_err());
}

#[test]
fn bridge_rejects_unbounded_configuration_and_oversized_descriptor() {
    let descriptor = EndpointDescriptor {
        label: "family".into(),
        projections: Vec::new(),
    };
    let zero_queue = EndpointBridgeLimits {
        queue_capacity: 0,
        ..EndpointBridgeLimits::default()
    };
    assert!(
        BridgeEndpoint::new(
            AuthenticatedPeerKey([1; 32]),
            descriptor.clone(),
            zero_queue
        )
        .is_err()
    );
    let tiny_message = EndpointBridgeLimits {
        max_message_bytes: 1,
        ..EndpointBridgeLimits::default()
    };
    assert!(BridgeEndpoint::new(AuthenticatedPeerKey([1; 32]), descriptor, tiny_message).is_err());
}

#[test]
fn admitted_request_keeps_transport_key_and_queue_full_fails_closed() {
    let session = chirograph::ProjectionSession("fixture-session".into());
    let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let wake_count = Arc::clone(&wakes);
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        wake_count.fetch_add(1, Ordering::SeqCst);
    });
    let (bridge, pump) = BridgeEndpoint::new_with_wake(
        AuthenticatedPeerKey([0x35; 32]),
        EndpointDescriptor {
            label: "family".into(),
            projections: Vec::new(),
        },
        EndpointBridgeLimits {
            queue_capacity: 1,
            ..EndpointBridgeLimits::default()
        },
        wake,
    )
    .unwrap();
    let bridge = std::sync::Arc::new(bridge);
    let (started, ready) = std::sync::mpsc::channel();
    let worker_bridge = std::sync::Arc::clone(&bridge);
    let worker_session = session.clone();
    let worker = std::thread::spawn(move || {
        started.send(()).unwrap();
        worker_bridge.call(EndpointCall::Resource(ResourceRequest {
            session: worker_session,
            resource: chirograph::ContentHash::of(b"first"),
        }))
    });
    ready.recv().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while wakes.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        wakes.load(Ordering::SeqCst),
        1,
        "first request was enqueued"
    );
    let full = bridge.call(EndpointCall::Resource(ResourceRequest {
        session: session.clone(),
        resource: chirograph::ContentHash::of(b"second"),
    }));
    assert!(full.unwrap_err().0.contains("queue is full"));
    assert_eq!(wakes.load(Ordering::SeqCst), 1, "full queue does not wake");
    let request = pump
        .try_next()
        .unwrap()
        .expect("first request stays queued");
    assert_eq!(request.peer_key.as_bytes(), &[0x35; 32]);
    request
        .respond(Ok(EndpointAnswer::Resource(ResourceResponse::new(
            session,
            b"resource".to_vec(),
        ))))
        .unwrap();
    assert!(matches!(
        worker.join().unwrap(),
        Ok(EndpointAnswer::Resource(_))
    ));
}

#[test]
fn wake_fires_after_enqueue_but_not_for_full_or_closed_queue() {
    let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let wake_count = Arc::clone(&wakes);
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        wake_count.fetch_add(1, Ordering::SeqCst);
    });
    let session = chirograph::ProjectionSession("wake-session".into());
    let (bridge, pump) = BridgeEndpoint::new_with_wake(
        AuthenticatedPeerKey([0x45; 32]),
        EndpointDescriptor {
            label: "family".into(),
            projections: Vec::new(),
        },
        EndpointBridgeLimits {
            queue_capacity: 1,
            ..EndpointBridgeLimits::default()
        },
        Arc::clone(&wake),
    )
    .unwrap();
    let bridge = Arc::new(bridge);
    let worker_bridge = Arc::clone(&bridge);
    let worker_session = session.clone();
    let worker = std::thread::spawn(move || {
        worker_bridge.call(EndpointCall::Resource(ResourceRequest {
            session: worker_session,
            resource: chirograph::ContentHash::of(b"queued"),
        }))
    });
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while wakes.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        wakes.load(Ordering::SeqCst),
        1,
        "successful enqueue wakes owner"
    );

    let full = bridge.call(EndpointCall::Resource(ResourceRequest {
        session: session.clone(),
        resource: chirograph::ContentHash::of(b"full"),
    }));
    assert!(full.unwrap_err().0.contains("queue is full"));
    assert_eq!(wakes.load(Ordering::SeqCst), 1, "full queue does not wake");

    let request = pump.try_next().unwrap().expect("first queued request");
    request
        .respond(Ok(EndpointAnswer::Resource(ResourceResponse::new(
            session.clone(),
            b"resource".to_vec(),
        ))))
        .unwrap();
    assert!(matches!(
        worker.join().unwrap(),
        Ok(EndpointAnswer::Resource(_))
    ));

    let (closed, closed_pump) = BridgeEndpoint::new_with_wake(
        AuthenticatedPeerKey([0x46; 32]),
        EndpointDescriptor {
            label: "family".into(),
            projections: Vec::new(),
        },
        EndpointBridgeLimits::default(),
        wake,
    )
    .unwrap();
    drop(closed_pump);
    let closed = closed.call(EndpointCall::Resource(ResourceRequest {
        session,
        resource: chirograph::ContentHash::of(b"closed"),
    }));
    assert!(closed.unwrap_err().0.contains("pump is closed"));
    assert_eq!(
        wakes.load(Ordering::SeqCst),
        1,
        "closed queue does not wake"
    );
}
