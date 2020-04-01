/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindgen::ir::{Path, Type};

pub fn mangle_path(path: &Path, generic_values: &[Type], mangle_separator: &Option<String>) -> Path {
    internal_mangle_path(path, generic_values, true, mangle_separator)
}

pub fn mangle_name(name: &str, generic_values: &[Type], mangle_separator: &Option<String>) -> String {
    internal_mangle_name(name, generic_values, true, mangle_separator)
}

fn internal_mangle_path(path: &Path, generic_values: &[Type], last_in_parent: bool, mangle_separator: &Option<String>) -> Path {
    let name = path.name();
    let mangled_name = internal_mangle_name(name, generic_values, last_in_parent, mangle_separator);
    Path::new(mangled_name)
}

fn internal_mangle_name(name: &str, generic_values: &[Type], last_in_parent: bool, mangle_separator: &Option<String>) -> String {
    if generic_values.is_empty() {
        return name.to_owned();
    }

    let mut mangled = name.to_owned();
    let separator = match mangle_separator {
        Some(s) => s.to_owned(),
        None => "_".to_string()
    };

    mangled.push_str(separator.as_str()); // <
    for (i, ty) in generic_values.iter().enumerate() {
        if i != 0 {
            mangled.push_str(&concat_separators(&separator, 2)); // ,
        }

        let is_last = i == generic_values.len() - 1;
        match *ty {
            Type::Path(ref generic) => {
                mangled.push_str(&internal_mangle_name(
                    generic.export_name(),
                    generic.generics(),
                    last_in_parent && is_last,
                    mangle_separator,
                ));
            }
            Type::Primitive(ref primitive) => {
                mangled.push_str(primitive.to_repr_rust());
            }
            Type::MutRef(..)
            | Type::Ref(..)
            | Type::ConstPtr(..)
            | Type::Ptr(..)
            | Type::Array(..)
            | Type::FuncPtr(..) => {
                panic!("Unable to mangle generic parameter {:?} for '{}'", ty, name);
            }
        }

        // Skip writing the trailing '>' mangling when possible
        if is_last && !last_in_parent {
            mangled.push_str(&concat_separators(&separator, 3)); // >
        }
    }

    mangled
}

fn concat_separators(separator: &str, number: u8) -> String {
    let mut result: String = "".to_string();
    for _ in 0..number {
        result += separator;
    }
    result
}

#[test]
fn generics() {
    use crate::bindgen::ir::{GenericPath, PrimitiveType};

    fn float() -> Type {
        Type::Primitive(PrimitiveType::Float)
    }

    fn path(path: &str) -> Type {
        generic_path(path, &vec![])
    }

    fn generic_path(path: &str, generics: &[Type]) -> Type {
        let path = Path::new(path);
        let generic_path = GenericPath::new(path, generics.to_owned());
        Type::Path(generic_path)
    }

    // Foo<f32> => Foo_f32
    assert_eq!(
        mangle_path(&Path::new("Foo"), &vec![float()], &None),
        Path::new("Foo_f32")
    );

    // Foo<Bar<f32>> => Foo_Bar_f32
    assert_eq!(
        mangle_path(&Path::new("Foo"), &vec![generic_path("Bar", &[float()])], &None),
        Path::new("Foo_Bar_f32")
    );

    // Foo<Bar> => Foo_Bar
    assert_eq!(
        mangle_path(&Path::new("Foo"), &[path("Bar")], &None),
        Path::new("Foo_Bar")
    );

    // Foo<Bar<T>> => Foo_Bar_T
    assert_eq!(
        mangle_path(&Path::new("Foo"), &[generic_path("Bar", &[path("T")])], &None),
        Path::new("Foo_Bar_T")
    );

    // Foo<Bar<T>, E> => Foo_Bar_T_____E
    assert_eq!(
        mangle_path(
            &Path::new("Foo"),
            &[generic_path("Bar", &[path("T")]), path("E")],
            &None,
        ),
        Path::new("Foo_Bar_T_____E")
    );

    // Foo<Bar<T>, Bar<E>> => Foo_Bar_T_____Bar_E
    assert_eq!(
        mangle_path(
            &Path::new("Foo"),
            &[
                generic_path("Bar", &[path("T")]),
                generic_path("Bar", &[path("E")]),
            ],
            &None,
        ),
        Path::new("Foo_Bar_T_____Bar_E")
    );
}
