use std::cell::Cell;

use cursive::theme::PaletteColor::HighlightText;
use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::utils::markup::StyledString;
use cursive::view::ViewWrapper;
use cursive::views::{DummyView, LinearLayout, PaddedView, ResizedView, TextView};
use cursive::{Printer, Vec2, View, XY};
use sudo::RunningAs;
use xpc_sys::csr::{csr_check, CsrConfig};
use xpc_sys::rs_sysctlbyname;

pub fn make_layout() -> LinearLayout {
    let bold = Style::from(Color::Light(BaseColor::White)).combine(Effect::Bold);
    let mut layout = LinearLayout::horizontal();

    let mut macos = StyledString::styled("macOS: ", bold);
    let osproductversion =
        unsafe { rs_sysctlbyname("kern.osproductversion").unwrap_or("".to_string()) };
    let osversion = unsafe { rs_sysctlbyname("kern.osversion").unwrap_or("".to_string()) };
    macos.append_plain(format!("{} ({})", osproductversion, osversion));

    if sudo::check() == RunningAs::Root {
        macos.append_styled(" (root)", bold.combine(Color::Light(BaseColor::Red)));
    }

    layout.add_child(ResizedView::with_full_width(TextView::new(macos)));

    // If granted CSR_ALLOW_UNTRUSTED_KEXTS, SIP is probably off
    let mut sip = StyledString::styled("SIP: ", bold);
    let sip_data = unsafe {
        format!(
            "{}",
            csr_check(CsrConfig::ALLOW_UNTRUSTED_KEXTS.bits()) != 0
        )
    };
    sip.append_plain(sip_data);

    layout.add_child(ResizedView::with_full_width(TextView::new(sip)));

    layout
}
