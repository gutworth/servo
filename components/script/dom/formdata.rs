/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::FormDataBinding;
use dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use dom::bindings::codegen::InheritTypes::FileCast;
use dom::bindings::codegen::UnionTypes::FileOrString;
use dom::bindings::codegen::UnionTypes::FileOrString::{eFile, eString};
use dom::bindings::error::{Fallible};
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::blob::Blob;
use dom::file::File;
use dom::htmlformelement::HTMLFormElement;
use servo_util::str::DOMString;
use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};

#[deriving(Clone)]
#[jstraceable]
#[must_root]
pub enum FormDatum {
    StringData(DOMString),
    FileData(JS<File>)
}

#[dom_struct]
pub struct FormData {
    reflector_: Reflector,
    data: DOMRefCell<HashMap<DOMString, Vec<FormDatum>>>,
    global: GlobalField,
    form: Option<JS<HTMLFormElement>>
}

impl FormData {
    fn new_inherited(form: Option<JSRef<HTMLFormElement>>, global: &GlobalRef) -> FormData {
        FormData {
            reflector_: Reflector::new(),
            data: DOMRefCell::new(HashMap::new()),
            global: GlobalField::from_rooted(global),
            form: form.map(|f| JS::from_rooted(f)),
        }
    }

    pub fn new(form: Option<JSRef<HTMLFormElement>>, global: &GlobalRef) -> Temporary<FormData> {
        reflect_dom_object(box FormData::new_inherited(form, global),
                           *global, FormDataBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef, form: Option<JSRef<HTMLFormElement>>) -> Fallible<Temporary<FormData>> {
        Ok(FormData::new(form, global))
    }
}

impl<'a> FormDataMethods for JSRef<'a, FormData> {
    #[allow(unrooted_must_root)]
    fn Append(self, name: DOMString, value: JSRef<Blob>, filename: Option<DOMString>) {
        let file = FormDatum::FileData(JS::from_rooted(self.get_file_from_blob(value, filename)));
        let mut data = self.data.borrow_mut();
        match data.entry(name) {
            Occupied(entry) => entry.into_mut().push(file),
            Vacant(entry) => {
                entry.set(vec!(file));
            }
        }
    }

    fn Append_(self, name: DOMString, value: DOMString) {
        let mut data = self.data.borrow_mut();
        match data.entry(name) {
            Occupied(entry) => entry.into_mut().push(FormDatum::StringData(value)),
            Vacant  (entry) => { entry.set(vec!(FormDatum::StringData(value))); },
        }
    }

    fn Delete(self, name: DOMString) {
        self.data.borrow_mut().remove(&name);
    }

    fn Get(self, name: DOMString) -> Option<FileOrString> {
        if self.data.borrow().contains_key_equiv(&name) {
            match (*self.data.borrow())[name][0].clone() {
                FormDatum::StringData(ref s) => Some(eString(s.clone())),
                FormDatum::FileData(ref f) => {
                    Some(eFile(f.clone()))
                }
            }
        } else {
            None
        }
    }

    fn Has(self, name: DOMString) -> bool {
        self.data.borrow().contains_key_equiv(&name)
    }
    #[allow(unrooted_must_root)]
    fn Set(self, name: DOMString, value: JSRef<Blob>, filename: Option<DOMString>) {
        let file = FormDatum::FileData(JS::from_rooted(self.get_file_from_blob(value, filename)));
        self.data.borrow_mut().insert(name, vec!(file));
    }

    fn Set_(self, name: DOMString, value: DOMString) {
        self.data.borrow_mut().insert(name, vec!(FormDatum::StringData(value)));
    }
}

trait PrivateFormDataHelpers{
  fn get_file_from_blob(&self, value: JSRef<Blob>, filename: Option<DOMString>) -> Temporary<File>;
}

impl PrivateFormDataHelpers for FormData {
    fn get_file_from_blob(&self, value: JSRef<Blob>, filename: Option<DOMString>) -> Temporary<File> {
        let global = self.global.root();
        let f: Option<JSRef<File>> = FileCast::to_ref(value);
        let name = filename.unwrap_or(f.map(|inner| inner.name().clone()).unwrap_or("blob".into_string()));
        File::new(&global.root_ref(), value, name)
    }
}
