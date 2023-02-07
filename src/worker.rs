use serde::{Deserialize, Serialize};
use yew_agent::{HandlerId, Public, WorkerLink};

use crate::wfc_field::Cell;

pub struct Worker {
    link: WorkerLink<Self>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerInput {
    pub idx: usize,
    pub len: usize,
    pub dim: usize,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerOutput {
    pub idx: usize,
    pub value: Box<[Cell]>,
}

impl yew_agent::Worker for Worker {
    type Input = WorkerInput;
    type Message = ();
    type Output = WorkerOutput;
    type Reach = Public<Self>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {
        // no messaging
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        // this runs in a web worker
        // and does not block the main
        // browser thread!

        let dim = msg.len;

        let data = (0..dim)
            .map(|_| Cell::new())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let output = Self::Output { value: data, idx: msg.idx };

        self.link.respond(id, output);
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}