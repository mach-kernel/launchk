use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::{Printer, Vec2, View, XY};
use xpc_sys::sysctlbyname_string;

use std::cell::Cell;
use xpc_sys::csr::sip_enabled;

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
            sysctlbyname_string("kern.osproductversion").unwrap_or("".to_string());
        let osversion = sysctlbyname_string("kern.osversion").unwrap_or("".to_string());
        let mac_os_data = format!("{} ({})", osproductversion, osversion);

        let sip_label = "SIP:";
        let sip_data = format!("{}", sip_enabled());

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
