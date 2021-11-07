use crate::components::nav_bar::NavBar;
use yew::{prelude::*, virtual_dom::VNode};

pub struct HomePage {}
pub enum Message {}

impl Component for HomePage {
    type Message = Message;
    type Properties = ();

    fn view(&self) -> VNode {
        html! {
            <>
                <NavBar/>
            </>
        }
    }

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
