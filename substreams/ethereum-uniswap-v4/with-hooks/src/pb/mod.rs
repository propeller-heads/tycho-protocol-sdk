// @generated
pub mod sf {
    // @@protoc_insertion_point(attribute:sf.substreams)
    pub mod substreams {
        include!("sf.substreams.rs");
        // @@protoc_insertion_point(sf.substreams)
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
            pub mod service {
                // @@protoc_insertion_point(attribute:sf.substreams.sink.service.v1)
                pub mod v1 {
                    include!("sf.substreams.sink.service.v1.rs");
                    // @@protoc_insertion_point(sf.substreams.sink.service.v1)
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
pub mod tycho {
    pub mod evm {
        // @@protoc_insertion_point(attribute:tycho.evm.v1)
        pub mod v1 {
            include!("tycho.evm.v1.rs");
            // @@protoc_insertion_point(tycho.evm.v1)
        }
    }
}
pub mod uniswap {
    // @@protoc_insertion_point(attribute:uniswap.v4)
    pub mod v4 {
        include!("uniswap.v4.rs");
        // @@protoc_insertion_point(uniswap.v4)
        // @@protoc_insertion_point(attribute:uniswap.v4.angstrom)
        pub mod angstrom {
            include!("uniswap.v4.angstrom.rs");
            // @@protoc_insertion_point(uniswap.v4.angstrom)
        }
    }
}
