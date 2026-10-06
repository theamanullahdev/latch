//! Left nav bound to the page stack.

use gtk::prelude::*;

pub fn build(stack: &gtk::Stack) -> gtk::StackSidebar {
    let sidebar = gtk::StackSidebar::new();
    sidebar.set_stack(stack);
    sidebar.set_size_request(180, -1);
    sidebar
}
