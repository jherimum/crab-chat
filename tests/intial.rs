//use libp2p::{Swarm, SwarmBuilder, swarm::dummy::Behaviour, tls, yamux};

// #[test]
// fn test_swarm() {
//     let mut swarm0 = create_swarm();
//     let mut swarm1 = create_swarm();

// }

// fn create_swarm() -> Swarm<Behaviour> {
//     SwarmBuilder::with_new_identity()
//         .with_tokio()
//         .with_tcp(Default::default(), tls::Config::new, yamux::Config::default)
//         .unwrap()
//         .with_behaviour(|_| Behaviour)
//         .unwrap()
//         .build()
// }
