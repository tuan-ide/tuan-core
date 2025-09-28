use crate::{ffi, graph_builders};
use crate::{
    graph::{Graph, NodeId},
    graph_builders::GraphBuilder as _,
};

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len);
    let p = v.as_mut_ptr();
    core::mem::forget(v);
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn dealloc(ptr: *mut u8, cap: usize) {
    unsafe {
        drop(Vec::from_raw_parts(ptr, 0, cap));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn create_graph() -> *mut Graph {
    let graph = Graph::new();
    Box::into_raw(Box::new(graph))
}

#[unsafe(no_mangle)]
pub extern "C" fn create_str(ptr: *const u8, len: usize) -> *mut ffi::Str {
    let s = ffi::Str { ptr, len };
    Box::into_raw(Box::new(s))
}

#[unsafe(no_mangle)]
pub extern "C" fn create_node(file_path: *mut ffi::Str) -> *mut ffi::Node {
    let file_path: String = unsafe { (*file_path).clone().into() };
    let path = std::path::PathBuf::from(file_path);
    let node = crate::graph::Node::from_path(path);
    Box::into_raw(Box::new(ffi::Node::from(&node)))
}

#[unsafe(no_mangle)]
pub extern "C" fn create_edge(from: NodeId, to: NodeId) -> *mut ffi::Edge {
    let edge = crate::graph::Edge { from, to };
    Box::into_raw(Box::new(ffi::Edge::from(&edge)))
}

#[unsafe(no_mangle)]
pub extern "C" fn add_node(graph_ptr: *mut Graph, node: *mut ffi::Node) {
    let graph = unsafe {
        assert!(!graph_ptr.is_null());
        &mut *graph_ptr
    };
    graph.add_node(unsafe { (*node).clone().into() });
}

#[unsafe(no_mangle)]
pub extern "C" fn add_edge(graph_ptr: *mut Graph, edge: *mut ffi::Edge) {
    let graph = unsafe {
        assert!(!graph_ptr.is_null());
        &mut *graph_ptr
    };
    graph.add_edge(unsafe { (*edge).clone().into() });
}

#[unsafe(no_mangle)]
pub extern "C" fn positioning(graph_ptr: *mut Graph) {
    let graph = unsafe {
        assert!(!graph_ptr.is_null());
        &mut *graph_ptr
    };
    graph.positioning();
}

#[unsafe(no_mangle)]
pub extern "C" fn display_graph(graph_ptr: *mut Graph) {
    let graph = unsafe {
        assert!(!graph_ptr.is_null());
        &mut *graph_ptr
    };
    println!("{:?}", graph);
}

#[unsafe(no_mangle)]
pub extern "C" fn graph_to_json(graph_ptr: *mut Graph) -> *mut ffi::Str {
    assert!(!graph_ptr.is_null());
    let graph = unsafe { &*graph_ptr };

    let json = serde_json::to_string(graph).unwrap();
    let bytes = json.into_bytes();
    let len = bytes.len();
    let ptr = bytes.as_ptr();

    core::mem::forget(bytes);

    let s = ffi::Str { ptr, len };
    Box::into_raw(Box::new(s))
}

#[unsafe(no_mangle)]
pub extern "C" fn free_str_handle(str_handle: *mut ffi::Str) {
    if str_handle.is_null() {
        return;
    }
    unsafe {
        let s = Box::from_raw(str_handle);
        let ptr = s.ptr as *mut u8;
        let len = s.len;
        drop(s);
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_graph_from_typescript_project(project_path: *mut ffi::Str) -> *mut Graph {
    let project_path: String = unsafe { (*project_path).clone().into() };
    let path = std::path::PathBuf::from(project_path);
    let builder = graph_builders::Typescript::new(path);
    let graph = builder.get_graph();
    Box::into_raw(Box::new(graph))
}
