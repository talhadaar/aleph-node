use core::result::Result;
use std::{marker::PhantomData, sync::Arc};

use aleph_primitives::BlockNumber;
use log::{debug, warn};
use sc_client_api::{Backend, Finalizer, HeaderBackend, LockImportRun};
use sp_blockchain::Error;
use sp_runtime::{
    traits::{Block, Header},
    Justification,
};

use crate::{BlockIdentifier, HashNum, IdentifierFor};

pub trait BlockFinalizer<BI: BlockIdentifier> {
    fn finalize_block(&self, block: BI, justification: Justification) -> Result<(), Error>;
}

pub struct AlephFinalizer<B, BE, C>
where
    B: Block,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    client: Arc<C>,
    phantom: PhantomData<(B, BE)>,
}

impl<B, BE, C> AlephFinalizer<B, BE, C>
where
    B: Block,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    pub(crate) fn new(client: Arc<C>) -> Self {
        AlephFinalizer {
            client,
            phantom: PhantomData,
        }
    }
}

impl<B, BE, C> BlockFinalizer<IdentifierFor<B>> for AlephFinalizer<B, BE, C>
where
    B: Block,
    B::Header: Header<Number = BlockNumber>,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    fn finalize_block(
        &self,
        block: IdentifierFor<B>,
        justification: Justification,
    ) -> Result<(), Error> {
        let HashNum { num: number, hash } = block;

        let status = self.client.info();
        if status.finalized_number >= number {
            warn!(target: "aleph-finality", "trying to finalize a block with hash {} and number {}
               that is not greater than already finalized {}", hash, number, status.finalized_number);
        }

        debug!(target: "aleph-finality", "Finalizing block with hash {:?} and number {:?}. Previous best: #{:?}.", hash, number, status.finalized_number);

        let update_res = self.client.lock_import_and_run(|import_op| {
            // NOTE: all other finalization logic should come here, inside the lock
            self.client
                .apply_finality(import_op, hash, Some(justification), true)
        });
        let status = self.client.info();
        debug!(target: "aleph-finality", "Attempted to finalize block with hash {:?}. Current best: #{:?}.", hash, status.finalized_number);
        update_res
    }
}
