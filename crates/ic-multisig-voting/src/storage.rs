// storage.rs - Storage trait and manager
use crate::Multisig;
use candid::{CandidType, Principal};

/// Trait for persisting multisig state
pub trait MultisigStorage<T> {
    type Error;

    /// Save multisig state
    fn save(&mut self, multisig: &Multisig<T>) -> Result<(), Self::Error>;

    /// Load multisig state (returns None if no state exists)
    fn load(&mut self) -> Result<Option<Multisig<T>>, Self::Error>;
}

/// No-op storage implementation (pure in-memory, no persistence)
#[derive(Clone, Debug, Default)]
pub struct NoStorage;

impl<T> MultisigStorage<T> for NoStorage {
    type Error = ();

    fn save(&mut self, _multisig: &Multisig<T>) -> Result<(), Self::Error> {
        Ok(()) // Do nothing
    }

    fn load(&mut self) -> Result<Option<Multisig<T>>, Self::Error> {
        Ok(None) // No persistence
    }
}

/// Multisig manager with automatic persistence
pub struct MultisigManager<T, S: MultisigStorage<T>> {
    multisig: Multisig<T>,
    storage: S,
}

impl<T, S> MultisigManager<T, S>
where
    T: CandidType + Clone,
    S: MultisigStorage<T>,
{
    /// Create manager with custom storage backend
    pub fn with_storage(
        owners: Vec<Principal>,
        threshold: u8,
        mut storage: S
    ) -> Result<Self, S::Error> {
        let multisig = match storage.load()? {
            Some(existing) => existing,
            None => Multisig::new(owners, threshold),
        };

        Ok(Self { multisig, storage })
    }

    /// Propose with automatic persistence
    pub fn propose(&mut self, caller: Principal, payload: T) -> Result<u64, String> {
        let result = self.multisig.propose(caller, payload)?;
        self.storage.save(&self.multisig)
            .map_err(|_| "storage error".to_string())?;
        Ok(result)
    }

    /// Approve with automatic persistence
    pub fn approve(&mut self, caller: Principal, id: u64) -> Result<Option<T>, String> {
        let result = self.multisig.approve(caller, id)?;
        self.storage.save(&self.multisig)
            .map_err(|_| "storage error".to_string())?;
        Ok(result)
    }

    /// Direct access to multisig for queries (no persistence needed)
    pub fn multisig(&self) -> &Multisig<T> {
        &self.multisig
    }

    /// Mutable access to multisig (caller responsible for saving)
    pub fn multisig_mut(&mut self) -> &mut Multisig<T> {
        &mut self.multisig
    }

    /// Manual save operation
    pub fn save(&mut self) -> Result<(), S::Error> {
        self.storage.save(&self.multisig)
    }

    /// Manual load operation
    pub fn load(&mut self) -> Result<(), S::Error> {
        if let Some(multisig) = self.storage.load()? {
            self.multisig = multisig;
        }
        Ok(())
    }
}

impl<T: CandidType + Clone> MultisigManager<T, NoStorage> {
    /// Create manager with no persistence (pure in-memory)
    pub fn in_memory(owners: Vec<Principal>, threshold: u8) -> Self {
        Self {
            multisig: Multisig::new(owners, threshold),
            storage: NoStorage,
        }
    }
}