use crossbeam_channel::{Receiver, Sender};
use micro_model_bot::{ComputeMove, MoveComputed};

pub fn listen(_opts: WSOpts) {
    todo!("Auth, and everything else")
}

pub struct WSOpts {
    pub compute_move_out: Receiver<ComputeMove>,
    pub move_computed_in: Sender<MoveComputed>,
}
impl WSOpts {
    pub fn from(c: &crate::registry::Components) -> Self {
        WSOpts {
            compute_move_out: c.compute_move_out.clone(),
            move_computed_in: c.move_computed_in.clone(),
        }
    }
}
