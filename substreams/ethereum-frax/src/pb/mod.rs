// @generated
pub mod contract {
    // @@protoc_insertion_point(attribute:contract.v1)
    pub mod v1 {
        include!("contract.v1.rs");
        // @@protoc_insertion_point(contract.v1)
    }
}
pub mod google {
    // @@protoc_insertion_point(attribute:google.protobuf)
    pub mod protobuf {
        include!("google.protobuf.rs");
        // @@protoc_insertion_point(google.protobuf)
    }
}
pub mod sf {
    // @@protoc_insertion_point(attribute:sf.substreams)
    pub mod substreams {
        include!("sf.substreams.rs");
        // @@protoc_insertion_point(sf.substreams)
        pub mod entity {
            // @@protoc_insertion_point(attribute:sf.substreams.entity.v1)
            pub mod v1 {
                include!("sf.substreams.entity.v1.rs");
                // @@protoc_insertion_point(sf.substreams.entity.v1)
            }
        }
        pub mod index {
            // @@protoc_insertion_point(attribute:sf.substreams.index.v1)
            pub mod v1 {
                include!("sf.substreams.index.v1.rs");
                // @@protoc_insertion_point(sf.substreams.index.v1)
            }
        }
        pub mod rpc {
            // @@protoc_insertion_point(attribute:sf.substreams.rpc.v2)
            pub mod v2 {
                include!("sf.substreams.rpc.v2.rs");
                // @@protoc_insertion_point(sf.substreams.rpc.v2)
            }
        }
        pub mod sink {
            pub mod database {
                // @@protoc_insertion_point(attribute:sf.substreams.sink.database.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.database.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.database.v1)
                }
            }
            pub mod service {
                // @@protoc_insertion_point(attribute:sf.substreams.sink.service.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.service.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.service.v1)
                }
            }
            pub mod sql {
                // @@protoc_insertion_point(attribute:sf.substreams.sink.sql.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.sql.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.sql.v1)
                }
            }
            pub mod subgraph {
                // @@protoc_insertion_point(attribute:sf.substreams.sink.subgraph.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.subgraph.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.subgraph.v1)
                }
            }
        }
        // @@protoc_insertion_point(attribute:sf.substreams.v1)
        pub mod v1 {
            include!("sf.substreams.v1.rs");
            // @@protoc_insertion_point(sf.substreams.v1)
        }
    }
}
