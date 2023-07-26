
use std::cell::RefCell;
use std::rc::{Weak, Rc};

use gtk_estate::gtk4::traits::{BoxExt, WidgetExt};
use gtk_estate::{HasObject, impl_has_application_window, impl_has_object, StateContainers}; //get_state_containers, 

use gtk_estate::gtk4::{self as gtk, Box, Orientation};

use gtk_estate::adw::{Application, ApplicationWindow, HeaderBar, WindowTitle, prelude::AdwApplicationWindowExt, gtk::prelude::ApplicationWindowExt, gtk::prelude::GtkWindowExt};

use gtk_estate::corlib::{NonOption, rc_self_setup}; //rc_self_refcell_setup, 

use crate::window_contents_state::WindowContentsState;

pub struct WindowState
{

    weak_self: RefCell<NonOption<Weak<Self>>>, //NonOption<Weak<RefCell<Self>>>,
    window: ApplicationWindow

}

impl WindowState
{

    pub fn new(app: &Application) ->  Rc<Self>//Rc<RefCell<Self>>
    {

        //setup GTK/Adw objects

        let window = ApplicationWindow::builder()
                .application(app)
                .default_width(1000)
                .default_height(1000)
                //.title("Escape It")
                //.show_menubar(true)
                //.content(&contents)
                .build();

        let this = Self
        {

            weak_self: RefCell::new(NonOption::invalid()),
            window

        };

        let rc_self =  Rc::new(this); //Rc::new(RefCell::new(this));

        //setup weak self reference

        //rc_self_refcell_setup!(rc_self, weak_self);

        rc_self_setup!(rc_self, weak_self);

        //get the state containers singletion

        let scs = StateContainers::get(); //get_state_containers();

        //add this application window

        scs.adw().borrow_mut_application_windows().add(&rc_self); //add_refcell(&rc_self);

        //add window contents

        WindowContentsState::new(&rc_self.window);

        rc_self.window.show();

        /*
        {

            //add window contents

            let rc_self_ref = rc_self.borrow();

            WindowContentsState::new(&rc_self_ref.window);

            rc_self_ref.window.show();

        }
        */

        //done!

        rc_self

    }

}

impl_has_application_window!(window, WindowState);

