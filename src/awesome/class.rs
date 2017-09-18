//! Utility methods and constructors for Lua classes

use std::sync::Arc;
use std::default::Default;
use std::borrow::BorrowMut;
use rlua::{self, Lua, ToLua, Table, MetaMethod, UserData, UserDataMethods,
           Value, AnyUserData};
use super::object::{self, Object};
use super::property::Property;

pub type Allocator = fn(&Lua) -> rlua::Result<Object>;
pub type Collector = fn(Object);
pub type PropF<'lua> = rlua::Function<'lua>;
pub type Checker = fn(&Object) -> bool;

#[derive(Debug)]
pub struct Class<'lua> {
    table: Table<'lua>
}

pub struct ClassState {
    name: String,
    // TODO Needed? If we store this like the object surely it's not needed...
    /// The global Lua key that corresponds to this class' signal list.
    /// Stored this way so that it's not tied to Lua lifetime.
    signals_global: Option<String>,
    // TODO Another storage as a key in lua...hmm
    parent: Option<String>,
    allocator: Option<Allocator>,
    collector: Option<Collector>,
    checker: Option<Checker>,
    instances: u32,
    /// The global Lua key that corresponds to this class' function.
    /// Stored this way so that it's not tied to Lua lifetime.
    index_miss_handler_global: Option<String>,
    /// The global Lua key that corresponds to this class' function.
    /// Stored this way so that it's not tied to Lua lifetime.
    newindex_miss_handler_global: Option<String>,
}

#[derive(Debug)]
pub struct ClassBuilder<'lua>(Class<'lua>);

// TODO Store lua as well so I don't have to pass it in everywhere...
impl <'lua> ClassBuilder<'lua> {
    pub fn method(self, lua: &Lua, name: String, meth: rlua::Function)
                  -> rlua::Result<Self> {
        let meta = self.0.table.get_metatable()
            .expect("Class had no meta table!");
        meta.set(name, meth)?;
        Ok(self)
    }

    pub fn property(self, prop: Property<'lua>) -> rlua::Result<Self> {
        let properties = self.0.table.get::<_, Table>("properties")?;
        let length = properties.len().unwrap_or(0) + 1;
        properties.set(length, prop);
        Ok(self)
    }

    pub fn save_class(mut self, lua: &'lua Lua, name: &str)
                      -> rlua::Result<Self> {
        lua.globals().set(name, self.0.table)?;
        self.0.table = lua.globals().get(name)?;
        Ok(self)
    }

    pub fn build(self) -> rlua::Result<Class<'lua>> {
        Ok(self.0)
    }
}

impl <'lua> ToLua<'lua> for Class<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.table.to_lua(lua)
    }
}

impl Default for ClassState {
    fn default() -> Self {
        ClassState {
            name: String::default(),
            signals_global: Option::default(),
            parent: Option::default(),
            allocator: Option::default(),
            collector: Option::default(),
            checker: Option::default(),
            instances: 0,
            index_miss_handler_global: Option::default(),
            newindex_miss_handler_global: Option::default(),

        }
    }
}

impl UserData for ClassState {}

impl <'lua> Class<'lua> {
    pub fn new(lua: &'lua Lua,
               allocator: Option<Allocator>,
               collector: Option<Collector>,
               checker: Option<Checker>)
               -> rlua::Result<ClassBuilder<'lua>> {
        let mut class = ClassState::default();
        class.allocator = allocator;
        class.collector = collector;
        class.checker = checker;
        let table = lua.create_table();
        table.set("data", class)?;
        table.set("index_miss_property",
                    lua.create_function(index_miss_property))?;
        table.set("newindex_miss_property",
                    lua.create_function(newindex_miss_property))?;
        table.set("properties", Vec::<Property>::new().to_lua(lua)?)?;
        let meta = lua.create_table();
        meta.set("signals", lua.create_table())?;
        // TODO Is this the correct indexing function? Hm.
        meta.set("__index", lua.create_function(object::default_index))?;
        // TODO __tostring
        table.set_metatable(Some(meta));
        Ok(ClassBuilder(Class { table }))
    }

    pub fn properties(&self) -> rlua::Result<Table<'lua>> {
        self.table.get("properties")
    }
}


// TODO Implement
// TODO return rlua::Value in result, however that will cause lifetime issues...
pub fn index_miss_property<'lua>(lua: &'lua Lua, obj: Table<'lua>)
                                 -> rlua::Result<Value<'lua>> {
    unimplemented!()
}
pub fn newindex_miss_property<'lua>(lua: &'lua Lua, obj: Table<'lua>)
                                    -> rlua::Result<Value<'lua>> {
    unimplemented!()
}

pub fn button_class(lua: &Lua) -> rlua::Result<Class> {
    let table = lua.globals().get::<_, Table>("__button_class")?;
    // TODO Assert is correct table
    Ok(Class { table })
}
