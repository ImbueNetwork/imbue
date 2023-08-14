use sp_runtime::DispatchError;

pub trait DisputeRaiser<AccountId>{

    fn raise_dispute(
        who: &AccountId,
        reason: &str,
        project_id: u32,
        
    ) -> Result<(), DispatchError>;
}

pub trait JurySelector{
     fn select_jury()-> Result<(), DispatchError>;
}