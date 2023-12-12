use rusty::commands::commands::*;
use gtk::prelude::*;
use gtk::*;
use std::cell::RefCell;
use std::rc::Rc;


const COMMIT_COMMAND_NAME: &str = "Commit";
const PULL_COMMAND_NAME: &str = "Pull";
const PUSH_COMMAND_NAME: &str = "Push";
const ADD_COMMAND_NAME: &str = "Add";
const REMOVE_COMMAND_NAME: &str = "Remove";
const CHECKOUT_COMMAND_NAME: &str = "Checkout";
const LOG_COMMAND_NAME: &str = "Log";
const BRANCH_COMMAND_NAME: &str = "Branch";
const STATUS_COMMAND_NAME: &str = "Status";

const HEIGHT_BUTTON: i32 = 68;
const WIDTH_BUTTON: i32 = 108;


fn main() {

    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Rusty User Interface")
            .default_width(1111)
            .default_height(HEIGHT_BUTTON * 9)
            .build();

        let paned = Paned::new(gtk::Orientation::Horizontal);
      
        let title_label = Label::new(Some("Title"));
        title_label.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);

        title_label.set_halign(gtk::Align::Start);
        title_label.set_markup("<span font_desc='20'> Rusty User Interface</span>");

        let output_label = Label::new(Some(" "));
        output_label.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);
        output_label.set_halign(gtk::Align::Start);
        let output_label_ref = Rc::new(RefCell::new(output_label.clone()));

        let actual_command = COMMIT_COMMAND_NAME;
        let actual_command_ref = Rc::new(RefCell::new(actual_command.clone()));

        let entry_buffer = EntryBuffer::new(None);
        let text_input = Entry::with_buffer(&entry_buffer);
        text_input.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);
        let text_input_ref = Rc::new(RefCell::new(text_input.clone()));
        text_input.set_placeholder_text(Some("Enter text here"));

        let button_right = Button::with_label("Commit");
        button_right.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);
        let button_right_ref = Rc::new(RefCell::new(button_right.clone()));

        let commit_button = Button::with_label(COMMIT_COMMAND_NAME);
        commit_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);
        commit_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().show();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = COMMIT_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(COMMIT_COMMAND_NAME);
        }});

        let pull_button = Button::with_label(PULL_COMMAND_NAME);
        pull_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);

        pull_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().hide();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = PULL_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(PULL_COMMAND_NAME);
        }});

        let push_button = Button::with_label(PUSH_COMMAND_NAME);
        push_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);

        push_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().hide();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = PUSH_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(PUSH_COMMAND_NAME);
        }});

        let add_button = Button::with_label(ADD_COMMAND_NAME);
        add_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);

        add_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().show();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = ADD_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(ADD_COMMAND_NAME);
        }});

        let remove_button = Button::with_label(REMOVE_COMMAND_NAME);
        remove_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);

        remove_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().show();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = REMOVE_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(REMOVE_COMMAND_NAME);
        }});

        let checkout_button = Button::with_label(CHECKOUT_COMMAND_NAME);
        checkout_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);

        checkout_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().show();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = CHECKOUT_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(CHECKOUT_COMMAND_NAME);
        }});

        let log_button = Button::with_label(LOG_COMMAND_NAME);
        log_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);

        log_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().show();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = LOG_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(LOG_COMMAND_NAME);
        }});

        let branch_button = Button::with_label(BRANCH_COMMAND_NAME);
        branch_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);
        
        branch_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().hide();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = BRANCH_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(BRANCH_COMMAND_NAME);
        }});

        let status_button = Button::with_label(STATUS_COMMAND_NAME);
        status_button.set_size_request(WIDTH_BUTTON, HEIGHT_BUTTON);

        status_button.connect_clicked({
            let button_right_ref = Rc::clone(&button_right_ref);
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let text_input_ref = Rc::clone(&text_input_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            text_input_ref.borrow_mut().set_text("");
            text_input_ref.borrow_mut().hide();
            output_label_ref.borrow_mut().set_text("");
            let mut actual_command_mut = actual_command_ref.borrow_mut();
            *actual_command_mut = STATUS_COMMAND_NAME;
            button_right_ref.borrow_mut().set_label(STATUS_COMMAND_NAME);
        }});

        button_right.connect_clicked({
            let actual_command_ref = Rc::clone(&actual_command_ref);
            let output_label_ref = Rc::clone(&output_label_ref);
            move |_| {
            
            let actual_command_mut = actual_command_ref.borrow_mut();
            let text = entry_buffer.text();

            let my_vec: Vec<&str> = vec![&text];
            let args: Option<Vec<&str>> = Option::from(my_vec);

            let result = match *actual_command_mut {
                CHECKOUT_COMMAND_NAME => Checkout::new().execute(args),
                ADD_COMMAND_NAME => Add::new().execute(args), 
                REMOVE_COMMAND_NAME => Rm::new().execute(args),
                COMMIT_COMMAND_NAME => {
                    let commit_vec: Vec<&str> = vec!["-m", &text];
                    let commit_args: Option<Vec<&str>> = Option::from(commit_vec);
                    Commit::new().execute(commit_args)
                }
                LOG_COMMAND_NAME => {
                    let log_vec: Vec<&str> = text.split_whitespace().collect();
                    let log_args: Option<Vec<&str>> = Option::from(log_vec);
                    Log::new().execute(log_args)
                }
                BRANCH_COMMAND_NAME => Branch::new().execute(None), 
                STATUS_COMMAND_NAME => Status::new().execute(None),
                PULL_COMMAND_NAME => Pull::new().execute(None),
                PUSH_COMMAND_NAME => Push::new().execute(None),
                _ => return
            };
            // Handle the result if needed
            output_label_ref.borrow_mut().set_markup("<span></span>");
            match result {
                Ok(mssg) => {
                    let  output_label_mut = output_label_ref.borrow_mut();
                    let success_message = format!("<span font_desc='20'>Command run successfully\n{}</span>", mssg);
                    output_label_mut.set_markup(&success_message);
                },
                Err(err) => {
                    let error_message = format!("<span font_desc='20'>Error during Command:\n{}</span>", err);
                    let  output_label_mut = output_label_ref.borrow_mut();
                    output_label_mut.set_markup(&error_message);
                }
        }}});


        let vbox = Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&title_label, false, false, 0);
        vbox.pack_start(&text_input, false, false, 0);
        vbox.pack_start(&button_right, false, false, 0);
        vbox.pack_start(&output_label, false, false, 0);

        let vbox2 = Box::new(gtk::Orientation::Vertical, 0);
        vbox2.pack_start(&commit_button, false, false, 0);
        vbox2.pack_start(&checkout_button, false, false, 0);
        vbox2.pack_start(&pull_button, false, false, 0);
        vbox2.pack_start(&push_button, false, false, 0);
        vbox2.pack_start(&remove_button, false, false, 0);
        vbox2.pack_start(&add_button, false, false, 0);
        vbox2.pack_start(&log_button, false, false, 0);
        vbox2.pack_start(&branch_button, false, false, 0);
        vbox2.pack_start(&status_button, false, false, 0);

        paned.pack1(&vbox2, false, false);
        paned.pack2(&vbox, false, false);

        window.add(&paned);
        paned.set_position(200);
        window.show_all();
    });

    application.run();
}
