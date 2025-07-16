use crate::pb::cowamm::{CowPoolBind, CowPoolBinds};
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::{Block};
use substreams_helper::hex::Hexable;


#[substreams::handlers::map]
pub fn map_cowpool_binds(block: Block) -> Result<CowPoolBinds> {
    const BIND_TOPIC: &str = "0xe4e1e53800000000000000000000000000000000000000000000000000000000";

    let cowpool_binds = block
    .logs()
    .filter(|log| {
        log.topics()
        .get(0)
        .map(|t| t.to_hex()) == Some(BIND_TOPIC.to_string())
    })
        .filter_map(|log| {
            let data = &log.data();
            let address = log.address().to_vec(); 
            if data.len() < 165 { return None; } 
            let token = data.get(80..100)?.to_vec();
            //the initial amount of tokens will be first bound to the pool, then subsequent increase or decreases will be achieved through exiting and joining the pool 
            // let liquidity = data.get(100..1nn)?.to_vec();
            let weight_bytes = data.get(132..164)?;
            Some(
               CowPoolBind {
                address: address,
                token,
                weight: weight_bytes.to_vec(), 
            })
        })
        .collect::<Vec<CowPoolBind>>();

     Ok(CowPoolBinds { binds: cowpool_binds })
}


// things to do :

// Should i add liquidity from the bind to the CowPoolBind {} ? The value in the bind itself is called "balance" and we need the data offset range 

//we wanted to use the dune table dune.cowprotocol.amm_lp_infos to get how they got the balance but seems like they extracted the table directly 

// how do we get the lp token balance change from the exit_pool and join_pool ? we also need to track the lp token balance change from each exit_pool adn join_pool operation how do we do this? query get supply? at every event? yeah possibly that get the supply at every event 

//now how does tycho work in terms of reserves? for each token do we need an initial reserve it needs for its attributes? so that in simulation it gets that for the liquidity calculation? or its from the balance delta now tha thting is the balance 

//deltas would start from the first exit or join pool operation, which is not the initial state of both pool tokens, we can get it from the bind balance from when it was initially bound 

//study the uni v2 protocols that use reserves from the sdk then relate it to the simulation - what do they use the attributes for 

// understand how ekubo works compared to these  

//in ethereum ekubo v2 , pancakeswap, the liquidity attribute is 0, probab;y the initial liquidity