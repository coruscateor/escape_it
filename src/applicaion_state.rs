use gtk::prelude::ApplicationExt;
use gtk_estate::gtk4 as gtk;

use gtk_estate::*;

use gtk_estate::adw::Application;

use std::{rc::*, any::Any};

use std::cell::{RefCell, Ref, RefMut};

use corlib::{NonOption, rc_self_setup}; //rc_self_refcell_setup,

use crate::window_state::*;

use tokio::runtime::{Runtime, Handle, Builder};

pub struct ApplicattionState
{

    app: Application,
    weak_self: RefCell<NonOption<Weak<Self>>>,
    tokio_rt: Runtime

}

impl ApplicattionState
{

    pub fn new(app: &Application) -> Rc<Self>
    {

        let tokio_rt = Builder::new_multi_thread().enable_all().build().expect("Tokio Runtime construction failed");

        //

        let this = Self
        {

            app: app.clone(),
            weak_self: RefCell::new(NonOption::invalid()),
            tokio_rt

        };
        
        let rc_self = Rc::new(this);

        rc_self_setup!(rc_self, weak_self);
        
        rc_self.app.connect_activate(move |app|
        {

            //new window

            WindowState::new(app);

        });

        let sc = StateContainers::get();

        sc.adw().borrow_mut_applications().add(&rc_self);

        rc_self

    }

    //get weak self

    //tokio_rt

    pub fn get_tokio_rt_handle_ref(&self) -> &Handle
    {

        self.tokio_rt.handle()

    }

    pub fn clone_tokio_rt_handle(&self) -> Handle
    {

        self.tokio_rt.handle().clone()

    }

}

impl_has_application!(app, ApplicattionState);

