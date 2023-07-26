use act_rs::{ActorFrontend, ActorState, HasInteractor, impl_mac_runtime_task_actor, DroppedIndicator, impl_default_on_enter_async, impl_default_on_exit_async, impl_default_on_enter_and_exit_async}; 

use act_rs::tokio::{RuntimeTaskActor, interactors::mspc::{SenderInteractor, channel}};

use act_rs::tokio::crossbeam::{NotfiyingArrayQueue, InputNotfiyingArrayQueue, OutputNotfiyingArrayQueue, get_notifying_array_queue_iowii, InputNotfiyingArrayQueueInteractor};

use tokio::sync::oneshot::Sender;

use escape_it_lib::escape_string;

use std::{marker::PhantomData, sync::Arc};

use tokio::runtime::{Runtime, Handle};

use act_rs::ActorInteractor;

pub enum ConversionActorMessage
{

    //get time?

    Convert(String, Sender<String>)

}

pub struct ConversionActorState
{

    input_queue_input_interactor: InputNotfiyingArrayQueueInteractor<Option<ConversionActorMessage>>, //Sender
    input_queue_output: OutputNotfiyingArrayQueue<Option<ConversionActorMessage>> //Reciver

}

impl ConversionActorState
{

    pub fn new() -> Self
    {

        let (input_queue_input_interactor, input_queue_output) = get_notifying_array_queue_iowii(5);

        Self
        {

            input_queue_input_interactor,
            input_queue_output

        }

    }

    impl_default_on_enter_and_exit_async!();

    async fn run_async(&mut self, di: &DroppedIndicator) -> bool
    {

        let input_val = self.input_queue_output.pop_or_wait().await;

        match input_val
        {

            Some(cam_val) =>
            {

                match cam_val
                {

                    ConversionActorMessage::Convert(input, returner) =>
                    {

                        let escaped = escape_string(&input, true);

                        if let Err(err) = returner.send(escaped)
                        {

                            eprintln!("Error {}", err);

                        }

                    }

                }

            },
            None => { /* Continue */ },
        }

        di.has_not_dropped()

    }


}

impl HasInteractor<InputNotfiyingArrayQueueInteractor<Option<ConversionActorMessage>>> for ConversionActorState
{

    fn get_interactor(&self) -> InputNotfiyingArrayQueueInteractor<Option<ConversionActorMessage>>
    {
        
        self.input_queue_input_interactor.clone()

    }

}

impl_mac_runtime_task_actor!(InputNotfiyingArrayQueueInteractor<Option<ConversionActorMessage>>, ConversionActorState, ConversionActor);
