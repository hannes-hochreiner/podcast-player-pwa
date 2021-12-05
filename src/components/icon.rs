use yew::prelude::*;

pub struct Icon {
    name: String,
    style: IconStyle,
}
pub enum Message {}

#[derive(Debug, Clone)]
pub enum IconStyle {
    Filled,
    Outlined,
}

#[derive(Clone, yew::Properties)]
pub struct IconProperties {
    pub name: String,
    pub style: IconStyle,
}

impl Component for Icon {
    type Message = Message;
    type Properties = IconProperties;

    fn view(&self) -> Html {
        match &self.style {
            &IconStyle::Filled => {
                html! {<span class="icon"><span class="material-icons">{&self.name}</span></span>}
            }
            &IconStyle::Outlined => {
                html! {<span class="icon"><span class="material-icons-outlined">{&self.name}</span></span>}
            }
        }
    }

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            name: props.name,
            style: props.style,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.name = props.name;
        self.style = props.style;
        true
    }
}
