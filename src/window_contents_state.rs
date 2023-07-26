
use std::cell::RefCell;

use std::rc::{Weak, Rc};

use std::time::Duration;

use gtk_estate::corlib::events::SenderEventFunc;

use gtk_estate::corlib::rc_default::RcDefault;

use gtk_estate::gtk4::glib::clone::Downgrade;

use gtk_estate::gtk4::traits::{BoxExt, WidgetExt, ButtonExt, TextViewExt, TextBufferExt};

use gtk_estate::{HasObject, impl_has_box, impl_has_object, StateContainers, RcSimpleTimeOut, SimpleTimeOut}; //get_state_containers, 

use gtk_estate::gtk4::{self as gtk, Box, Orientation, Label, BaselinePosition, Align, CenterBox, TextView, Button};

use gtk_estate::adw::{Application, ApplicationWindow, HeaderBar, WindowTitle, prelude::AdwApplicationWindowExt, gtk::prelude::ApplicationWindowExt, gtk::prelude::GtkWindowExt};

use gtk_estate::corlib::{NonOption, rc_self_setup}; //, rc_self_refcell_setup};

use gtk_estate::time_out::*;

//use gtk_estate::adw::{TabBar, TabPage, TabView};

use tokio::runtime::{Runtime, Handle, Builder};

use gtk_estate::gtk4::glib::object::Cast;

use gtk_estate::helpers::{widget_ext::{set_hvexpand_t, set_margin_start_and_end, set_margin_all}, text_view::get_text_view_string};

use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::error::TryRecvError;

use crate::applicaion_state::ApplicattionState;

use crate::conversion_actor::{ConversionActor, ConversionActorState, ConversionActorMessage};

use act_rs::ActorFrontend;

pub struct WindowContentsState
{

    weak_self: RefCell<NonOption<Weak<Self>>>,
    contents_box: Box,
    //app_window: ApplicationWindow,
    window_title: WindowTitle,
    hb: HeaderBar,
    tokio_rt_handle: Handle,
    conversion_actor: ConversionActor,
    conversion_job: RefCell<Option<Receiver<String>>>,
    conversion_job_timeout: RcSimpleTimeOut,
    input_text: TextView,
    output_text: TextView

}

impl WindowContentsState
{

    pub fn new(app_window: &ApplicationWindow) -> Rc<Self>
    {

        let contents_box = Box::new(Orientation::Vertical, 0);

        contents_box.set_vexpand(true);

        let window_title = WindowTitle::new("Escape It", "");

        let hb = HeaderBar::builder().title_widget(&window_title).build();

        contents_box.append(&hb);

        //Add contents:

        //Top CenterBox

        let input_cbox = CenterBox::new();

        let input_label = Label::new(Some("Input:"));

        input_cbox.set_start_widget(Some(&input_label));

        contents_box.append(&input_cbox);

        //Top/Middle TextView

        let input_text = TextView::new();

        set_hvexpand_t(&input_text);

        set_margin_all(&input_text, 2);

        contents_box.append(&input_text);

        //Middle CenterBox

        let output_cbox = CenterBox::new();

        let output_label = Label::new(Some("Output:"));

        output_cbox.set_start_widget(Some(&output_label));

        //Convert button

        let convert_button = Button::new();

        convert_button.set_label("Convert");

        output_cbox.set_center_widget(Some(&convert_button));

        //
        
        contents_box.append(&output_cbox);

        //Bottom TextView

        let output_text = TextView::new();

        contents_box.append(&output_text);

        set_hvexpand_t(&output_text);

        set_margin_all(&output_text, 2);

        //Get Tokio handle:

        let scs = StateContainers::get();

        let tokio_rt_handle;
        
        {

            let application = app_window.application().unwrap();
    
            let adw_application = application.downcast_ref::<Application>().unwrap();
    
            let applications = scs.adw().borrow_applications();

            let app_state = applications.get(&adw_application).unwrap();
    
            let app_state_ref = app_state.downcast_ref::<ApplicattionState>().unwrap();
    
            tokio_rt_handle = app_state_ref.clone_tokio_rt_handle();

        }

        let conversion_actor_state = ConversionActorState::new();

        let conversion_actor = ConversionActor::new(&tokio_rt_handle, conversion_actor_state);

        let this = Self
        {

            weak_self: NonOption::invalid_rfc(), //invalid_refcell(),
            contents_box,
            //app_window: app_window.clone(),
            window_title,
            hb,
            tokio_rt_handle,
            conversion_actor,
            conversion_job: RefCell::new(None),
            conversion_job_timeout: SimpleTimeOut::new(Duration::new(1, 0)),
            input_text,
            output_text

        };

        let rc_self = Rc::new(this);

        //setup weak self reference

        rc_self_setup!(rc_self, weak_self);

        /* 
        //Add contents:

        //Top CenterBox

        let input_cbox = CenterBox::new();

        let input_label = Label::new(Some("Input:"));

        input_cbox.set_start_widget(Some(&input_label));

        rc_self.contents_box.append(&input_cbox);

        //rc_self.contents_box.append(

        //Top/Middle TextView

        let input_text = TextView::new();

        set_hvexpand_t(&input_text);

        set_margin_all(&input_text, 2);

        //input_text.set_margin_start(2);

        //input_text.set_margin_end(2);

        rc_self.contents_box.append(&input_text);

        //Middle CenterBox

        let output_cbox = CenterBox::new();

        let output_label = Label::new(Some("Output:"));

        output_cbox.set_start_widget(Some(&output_label));

        //Convert button

        let convert_button = Button::new();

        convert_button.set_label("Convert");
        */

        let weak_self = rc_self.downgrade();

        convert_button.connect_clicked(move |_btn|
        {

            if let Some(this) = weak_self.upgrade()
            {

                if this.conversion_job.borrow().is_some()
                {

                    return;

                }

                let input = get_text_view_string(&this.input_text);

                let interactor = this.conversion_actor.get_interactor_ref();

                let (sender, reciver) = tokio::sync::oneshot::channel();

                if let Err(_err) = interactor.get_queue_ref().push_notify_one(Some(ConversionActorMessage::Convert(input, sender)))
                {

                    this.output_text.buffer().set_text("ConversionActor input queue is full");

                }
                else
                {

                    *this.conversion_job.borrow_mut() = Some(reciver);

                    this.conversion_job_timeout.start();
                    
                }

            }

        });

        let weak_self = rc_self.downgrade();

        rc_self.conversion_job_timeout.set_function(move |_sto|
        {

            if let Some(this) = weak_self.upgrade()
            {

                let mut conversion_job_timeout_mut = this.conversion_job.borrow_mut();

                //let successful;

                if let Some(rec) = conversion_job_timeout_mut.as_mut() //.take()
                {

                    match rec.try_recv()
                    {

                        Ok(res) =>
                        {

                            this.output_text.buffer().set_text(res.as_str());

                            //successful = true;

                        },
                        Err(err) =>
                        {

                            if let TryRecvError::Closed = err
                            {

                                this.output_text.buffer().set_text(err.to_string().as_str());

                                //return false;
                                
                            }
                            else
                            {

                                //Empty - Try again soon

                                return true;
                                
                            }

                            //error
                            
                            //successful = false;

                        }

                    }

                    *conversion_job_timeout_mut = None;

                    //return successful;

                }

            }

            false

        });

        /*
        output_cbox.set_center_widget(Some(&convert_button));

        

        //
        
        rc_self.contents_box.append(&output_cbox);

        //Bottom TextView

        let output_text = TextView::new();

        rc_self.contents_box.append(&output_text);

        set_hvexpand_t(&output_text);

        set_margin_all(&output_text, 2);
        */

        //Add to StateContainers

        scs.gtk().borrow_mut_boxes().add(&rc_self);

        app_window.set_content(Some(&rc_self.contents_box));

        rc_self

    }

}

impl_has_box!(contents_box, WindowContentsState);

