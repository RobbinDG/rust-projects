use iced_aw::{badge, style, Badge};

pub fn bool_badge<M>(value: bool) -> Badge<'static, M> {
    if value {
        badge("true").style(style::badge::success).padding(2)
    } else {
        badge("false").style(style::badge::danger).padding(2)
    }
}