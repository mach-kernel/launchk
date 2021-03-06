use std::cell::Cell;

use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::{Printer, Vec2, View, XY};
use xpc_sys::csr::{csr_check, CsrConfig};
use xpc_sys::rs_sysctlbyname;

pub struct SysInfo {
    current_size: Cell<XY<usize>>,
}

impl Default for SysInfo {
    fn default() -> Self {
        Self {
            current_size: Cell::new(XY::new(0, 0)),
        }
    }
}

impl View for SysInfo {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let middle = self.current_size.get().x / 2;

        let mac_os_label = "macOS:";
        let osproductversion =
            unsafe { rs_sysctlbyname("kern.osproductversion").unwrap_or("".to_string()) };
        let osversion = unsafe { rs_sysctlbyname("kern.osversion").unwrap_or("".to_string()) };
        let mac_os_data = format!("{} ({})", osproductversion, osversion);

        // If granted CSR_ALLOW_UNTRUSTED_KEXTS, SIP is probably off
        let sip_label = "SIP:";
        let sip_data = unsafe {
            format!(
                "{}",
                csr_check(CsrConfig::ALLOW_UNTRUSTED_KEXTS.bits()) != 0
            )
        };

        let bold = Style::from(Color::Light(BaseColor::White));
        let bold = bold.combine(Effect::Bold);

        printer.with_style(bold, |p| p.print(XY::new(0, 0), mac_os_label));
        printer.print(XY::new(mac_os_label.chars().count() + 1, 0), &mac_os_data);

        printer.with_style(bold, |p| p.print(XY::new(middle, 0), sip_label));
        printer.print(
            XY::new(middle + sip_label.chars().count() + 1, 0),
            &sip_data,
        );
    }

    fn layout(&mut self, size: Vec2) {
        self.current_size.replace(size);
    }

    fn required_size(&mut self, constraint: XY<usize>) -> XY<usize> {
        XY::new(constraint.x, 1)
    }
}
