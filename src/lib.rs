mod canvas;
mod wfc_field;
mod types;
pub mod worker;
use crate::canvas::Canvas;

use yew::prelude::*;


pub struct App {
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self { }
    }

    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <Canvas/>
            </>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
