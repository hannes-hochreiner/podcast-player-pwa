use yew::prelude::*;

pub struct Icon {}
pub enum Message {}

#[derive(Debug, Clone, PartialEq)]
pub enum IconStyle {
    Filled,
    Outlined,
}

#[derive(Clone, yew::Properties, PartialEq)]
pub struct IconProperties {
    pub name: String,
    pub style: IconStyle,
}

impl Component for Icon {
    type Message = Message;
    type Properties = IconProperties;

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &ctx.props().style {
            &IconStyle::Filled => {
                html! {<span class="icon"><span class="material-icons">{ctx.props().name.clone()}</span></span>}
            }
            &IconStyle::Outlined => {
                html! {<span class="icon"><span class="material-icons-outlined">{ctx.props().name.clone()}</span></span>}
            }
        }
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }
}
