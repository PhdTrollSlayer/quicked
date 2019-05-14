use gtk::prelude::*;

use image::{self, imageops};

use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

fn main() {
    gtk::init().unwrap();

    let glade_src = "layout.glade";

    let builder = gtk::Builder::new_from_file(glade_src);

    let janela: gtk::Window = builder.get_object("janela").unwrap();
    let preview_window: Rc<RefCell<gtk::Window>> =
        Rc::new(RefCell::new(builder.get_object("preview_window").unwrap()));
    let close_button: gtk::Button = builder.get_object("close_preview").unwrap();

    let escolha1: gtk::Button = builder.get_object("escolha1").unwrap();
    let arquivo1: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));
    let escolha2: gtk::FileChooser = builder.get_object("escolha2").unwrap();
    let arquivo2 = Rc::new(RefCell::new(String::new()));
    let merge_button: gtk::Button = builder.get_object("merge").unwrap();
    let preview_button: gtk::Button = builder.get_object("preview").unwrap();

    let nome1: Rc<RefCell<gtk::Label>> =
        Rc::new(RefCell::new(builder.get_object("nome1").unwrap()));
    let nome2: Rc<RefCell<gtk::Label>> =
        Rc::new(RefCell::new(builder.get_object("nome2").unwrap()));
    let escolha_final: gtk::Button = builder.get_object("escolhafinal").unwrap();
    let nomefinal: Rc<RefCell<String>> =
        Rc::new(RefCell::new(String::new()));
    let imagem_final: Rc<RefCell<gtk::Image>> =
        Rc::new(RefCell::new(builder.get_object("preview_image").unwrap()));

    janela.show_all();
    janela.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let u = (
        arquivo1.clone(),
        arquivo2.clone(),
        nome1.clone(),
        nome2.clone(),
        nomefinal.clone(),
        imagem_final.clone(),
        preview_window.clone(),
    );

    close_button.connect_clicked(clone! (u => move |_| {
        u.6.borrow().hide();
    }));

    let j = janela.clone();
    escolha1.connect_clicked(clone! (u => move |_| {
        let fc = gtk::FileChooserDialog::new(
            Some("Abrir pasta"), Some(&j), gtk::FileChooserAction::SelectFolder);
        fc.add_buttons(&[
            ("Selecionar", gtk::ResponseType::Ok.into()),
            ("Cancelar", gtk::ResponseType::Cancel.into()),
        ]);

        if fc.run() == gtk::ResponseType::Ok.into(){
            let folder = fc.get_filename().unwrap();
            u.2.borrow().set_label(folder.to_str().unwrap());

            for e in fs::read_dir(folder).unwrap() {
                let e = e.unwrap();
                let path = e.path();
                u.0.borrow_mut().push(path);
            }
        }

        fc.destroy();
    }));
    escolha_final.connect_clicked(clone! (u => move |b| {
        u.4.borrow_mut().clear();

        let fc = gtk::FileChooserDialog::new(
            Some("Abrir pasta"), Some(&janela), gtk::FileChooserAction::SelectFolder);
        fc.add_buttons(&[
            ("Selecionar", gtk::ResponseType::Ok.into()),
            ("Cancelar", gtk::ResponseType::Cancel.into()),
        ]);

        if fc.run() == gtk::ResponseType::Ok.into(){
            let folder = fc.get_filename().unwrap();
            b.set_label(folder.to_str().unwrap());

            u.4.borrow_mut().push_str(folder.to_str().unwrap());
        }

        fc.destroy();
    }));

    escolha2.connect_update_preview(clone! (u => move |f| {
        u.1.borrow_mut().clear();
        u.1.borrow_mut().push_str(match f.get_preview_filename(){
                                    Some(b) => String::from(b.to_str().expect("erro 2")),
                                    None => String::from("/"),
        }.as_str());

        let p = String::from(u.1.borrow().clone());
        u.3.borrow().set_label(&p);
    }));

    preview_button.connect_clicked(clone! (u => move |_| {
        let mut img1 = image::open(u.0.borrow_mut().pop().clone().unwrap().as_path()).unwrap();
        let mut img2 = image::open(Path::new(&u.1.borrow().clone())).unwrap();

        img2 = img2.resize_exact(img1.to_rgba().width(), img2.to_rgba().height(), image::FilterType::Nearest);

        let top = img1.to_rgba().height() - img2.to_rgba().height();

        imageops::overlay(&mut img1, &img2, 0, top);

        let mut tmp = File::create("tmp.png").unwrap();
        img1.resize_exact(500, 500, image::FilterType::Nearest).write_to(&mut tmp, image::ImageOutputFormat::PNG).unwrap();

        u.5.borrow().set_from_file(Path::new("tmp.png"));
        std::fs::remove_file("tmp.png").unwrap();
        preview_window.borrow().show_all();
    }));

    merge_button.connect_clicked(clone! (u => move |_| {
            let final_path = PathBuf::from(u.4.borrow().clone());
            println!("Salvando em: {:?}", final_path);
            
            if final_path.is_dir() {
                for i in u.0.borrow().iter() {
                    let mut img1 = image::open(i).unwrap();
                    let mut img2 = image::open(Path::new(&u.1.borrow().clone())).unwrap();

                    img2 = img2.resize_exact(img1.to_rgba().width(), img2.to_rgba().height(), image::FilterType::Nearest);

                    let top = img1.to_rgba().height() - img2.to_rgba().height();

                    imageops::overlay(&mut img1, &img2, 0, top);
                    let mut fp = PathBuf::new();
                    fp.push(final_path.to_str().unwrap());
                    fp.push(i.file_name().unwrap().to_str().unwrap());

                    let mut buffer = File::create(fp.clone()).unwrap();
                    img1.write_to(&mut buffer, image::ImageOutputFormat::PNG).unwrap();
                    println!("Sallvo: {:?}", fp);
                }
            }
       }));

    gtk::main();
}
