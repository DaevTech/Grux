use grux::grux_port_manager::PortManager;

#[tokio::test]
async fn test_port_allocation() {
    let manager = PortManager::new(9000, 9002);

    // Test basic allocation
    let port1 = manager.allocate_port("service1".to_string()).await;
    assert_eq!(port1, Some(9000));

    let port2 = manager.allocate_port("service2".to_string()).await;
    assert_eq!(port2, Some(9001));

    let port3 = manager.allocate_port("service3".to_string()).await;
    assert_eq!(port3, Some(9002));

    // Should return None when no more ports available
    let port4 = manager.allocate_port("service4".to_string()).await;
    assert_eq!(port4, None);
}

#[tokio::test]
async fn test_port_release_and_reuse() {
    let manager = PortManager::new(9000, 9001);

    // Allocate all ports
    let port1 = manager.allocate_port("service1".to_string()).await;
    let port2 = manager.allocate_port("service2".to_string()).await;
    assert_eq!(port1, Some(9000));
    assert_eq!(port2, Some(9001));

    // No more ports available
    let port3 = manager.allocate_port("service3".to_string()).await;
    assert_eq!(port3, None);

    // Release a port
    manager.release_port(9000).await;

    // Should be able to reuse the released port
    let port4 = manager.allocate_port("service4".to_string()).await;
    assert_eq!(port4, Some(9000));
}

#[tokio::test]
async fn test_release_all_ports_for_service() {
    let manager = PortManager::new(9000, 9002);

    // Allocate ports to different services
    manager.allocate_port("service1".to_string()).await;
    manager.allocate_port("service1".to_string()).await;
    manager.allocate_port("service2".to_string()).await;

    // Release all ports for service1
    let released = manager.release_all_ports_for_service("service1").await;
    assert_eq!(released.len(), 2);

    // Should be able to allocate new ports now
    let port = manager.allocate_port("service3".to_string()).await;
    assert!(port.is_some());
}

#[tokio::test]
async fn test_singleton_manager() {
    let manager = PortManager::instance();

    let port = manager.allocate_port("php-worker-1".to_string()).await;
    assert_eq!(port, Some(9000));

    let available_count = manager.available_port_count().await;
    assert_eq!(available_count, 1000); // 9000-10000 = 1001 ports, 1 allocated

    // Test that multiple calls return the same instance
    let manager2 = PortManager::instance();
    let port2 = manager2.allocate_port("php-worker-2".to_string()).await;
    assert_eq!(port2, Some(9001));

    // Clean up
    manager.release_port(9000).await;
    manager.release_port(9001).await;
}
