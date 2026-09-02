use std::io::Write;

use super::node_utils::NodeType;

/*
    nodes identified by index in a large, flat array
    using u32 rather than usize to cut down on bit width, in an attempt to improve efficiency
    does introduce a ~4GiB (it's slightly more than 4 Gibibytes) size limit on number of nodes though

    note that to change this, would need to go hunting in several different files - it's a choice whose consequences have disseminated throughout the codebase
*/
pub type NodeId = u32;

/*
    used as the child of some nodes to signify that there is no node, and is used to terminate each child list
    NodeId::MAX is the last valid node identifier
    using this because if reaching this limit, going to have a problem anyways
*/
pub const NO_NODE: NodeId = NodeId::MAX;

/*
    the largest source size this parser accepts, imposed by the choice of u32, and needing to express the end of the input as a u32
    need usize because value is being compared to input string length
*/
pub const MAX_SOURCE_LEN: usize = NodeId::MAX as usize - 1;

#[derive(Debug, Clone, Copy)]
/// Each element in the produced AST is of this type, where each node specifies some node type over a range of characters in the input string
pub struct Node {
    pub node_type: NodeType,

    // byte offsets in the input string
    pub start: u32,
    pub end: u32,

    // members to allow linked list behaviours in the array of nodes

    // NO_NODE when this node has no child nodes
    pub first_child: NodeId,

    // reference to last child for constant time appending
    pub last_child: NodeId,

    // NO_NODE when this node is the last of its parent's child nodes
    pub next_sibling: NodeId,
}

#[derive(Debug)]
/// The AST produced by the parser is rooted by a NodeId, with parsed content stored as an array of nodes.
/// These nodes are stored - conceptually - as a linked list, allowing one array for the whole tree
pub struct AST {
    pub root: NodeId,
    pub nodes: Vec<Node>,
}

// method implementations for AST struct
impl AST {
    /// Initialises the flat array for storing the nodes in the AST.
    /// Is created with capacity based on a guess as to how many input characters per AST node
    pub(crate) fn init_with_capacity(source_len: usize) -> Self {
        /*
            Takes the length of the input string, and returns an AST struct
            (ie. the Self, with a capital S, is an alias for the type the enclosing impl block is for)

            For clarity: Self, with a capital S, is a type - ie. an alias for AST
                         self, with a lower case s, is a value of type AST
        */

        AST {
            root: NO_NODE,                                  // initially, no node is set as root
            nodes: Vec::with_capacity(source_len / 8 + 16), // guesses that the mean number of chars per node will be 8. Allocs another 16 for spare. Values tuned empirically over a few varied benchmarking files
        }
    }

    #[inline(always)]
    /// Takes a borrowed AST, and a NodeId, and returns that node (borrowed).
    /// ie. Essentially dereferences a NodeId
    pub(crate) fn get_node(&self, node_id: NodeId) -> &Node {
        &self.nodes[node_id as usize] // need to cast id (u32) to usize, for valid array access
    }

    #[inline(always)]
    /// Takes a borrowed AST, and a NodeId, and returns that node (borrowed, but mutable).
    /// ie. Essentially dereferences a NodeId, with mutations to the node allowed
    pub(crate) fn get_mut_node(&mut self, node_id: NodeId) -> &mut Node {
        &mut self.nodes[node_id as usize]
    }

    #[inline(always)]
    /// Takes a node type, and creates a new node of that type in the AST.
    /// The range of input characters the node binds must also be supplied.
    /// However, the node created is detatched - setting its parent, next sibling, and child(ren) must be done separately
    pub(crate) fn new_detatched_node(
        &mut self,
        node_type: NodeType,
        start: u32,
        end: u32,
    ) -> NodeId {
        // note that the borrowed AST must be mutable, as the AST's nodes array is being modified

        // check to ensure panic on (extremely unlikely in real-world use) scenario of node count limit being reached
        // prevents quiet corruption
        assert!(
            self.nodes.len() < NO_NODE as usize,
            "AST node count limit reached. The last usable NodeId is {} (because NO_NODE takes NodeId::MAX)",
            NO_NODE - 1
        );

        // the new node's id is simply the number of existing nodes (because of the zero-based indexing)
        let node_id = self.nodes.len() as NodeId;

        // push the node to the array
        self.nodes.push(Node {
            node_type,
            start,
            end,
            first_child: NO_NODE,
            last_child: NO_NODE,
            next_sibling: NO_NODE,
        });

        // ownership goes to method caller
        node_id
    }

    #[inline(always)]
    /// Takes an AST, and two NodeID's. The second node is appended as the last child of the first node.
    /// The [`next_sibling`] link of the (now) child node is cleared. Thus, this can be used to move a child into a different node's child array
    pub(crate) fn append_child_to_parent(&mut self, parent: NodeId, child: NodeId) {
        // note that the borrowed AST must be mutable, as the AST's nodes array is being modified

        // clear sibling link of child node - prevents it referencing nodes in other lists
        // (needs a mutable version of the node, because it is modifying it)
        self.get_mut_node(child).next_sibling = NO_NODE;

        // get the last node of the parent node
        let last = self.get_node(parent).last_child;

        // handle siblings of the list the child node is being appended to
        if last == NO_NODE {
            // the parent node had no children, so this is simply the first element in the list
            self.get_mut_node(parent).first_child = child;
        } else {
            // parent node had at least one child node
            // need to set the next sibling of the last of these to point to the newly appended child node
            self.get_mut_node(last).next_sibling = child;
        }

        // list references handled, now append the child node to the parent's child list
        self.get_mut_node(parent).last_child = child;
    }

    #[inline]
    /// Takes an AST and a NodeId, and removes all children from that node.
    /// The list of removed children is returned
    pub(crate) fn take_child_nodes(&mut self, parent: NodeId) -> NodeId {
        // get the first child node of the parent node
        let first = self.get_node(parent).first_child;

        // get a mutable reference to the parent node, so that the node's children can be removed from the node
        let parent = self.get_mut_node(parent);

        // remove child nodes
        parent.first_child = NO_NODE;
        parent.last_child = NO_NODE;

        // return first element in the child node list, which on account of the list references in each node, provides access to the whole (now orphaned) child node list
        first
    }

    // PRINT OUT LOGIC

    /// Takes an AST, and the input string (borrowed). Writes out AST to standard output
    /// The whole tree is written into a buffer first and put out with a single call on a locked handle,
    /// so redirecting to a file with `>` costs one write rather than one per node
    pub fn ast_to_stdout(&self, input_string: &str) {
        /*
            note that for portability with windows console, output must be UTF-8
            this is checked in the to-AST implementation, as String and &str are guaranteed to refer to valid UTF-8
            the to-HTML functionality also ensures that the output is UTF-8
        */

        // create output array
        let mut output_array = Vec::new();

        // populate output array
        self.ast_to_array(input_string, &mut output_array);

        // get access to process' standard output, and lock it so that the write can proceed in one go
        // lock is released as function goes out of scope
        let mut stdout = std::io::stdout().lock();

        // so .write_all() can be used below
        use std::io::Write;

        // if error is broken pipe (eg. downstream program exited early), don't want to error
        // otherwise, write error to standard error, not standard output (as standard output may be what failed)
        if let Err(error) = stdout.write_all(&output_array)
            && error.kind() != std::io::ErrorKind::BrokenPipe
        {
            eprintln!("Failed to write AST: {error}");
        }
    }

    /// Takes an AST and the input string (borrowed), and appends a printable form of the tree to `out`.
    pub fn ast_to_array(&self, input_string: &str, output_array: &mut Vec<u8>) {
        // printable AST is significantly larger than input, due to the range captured by each node being printed
        output_array.reserve(input_string.len() * 4 + 256);

        self.print_node(self.root, input_string, 0, output_array);
    }

    /// Takes an AST, a NodeId, the input string (borrowed), a node depth (for printing nodes with indentation), and the buffer to write to. Appends node information to the buffer.
    /// Recursively calls itself until AST has been exhausted
    fn print_node(
        &self,
        node_id: NodeId,
        input_string: &str,
        node_depth: usize,
        output_array: &mut Vec<u8>,
    ) {
        /*
            self.get_node() hands back an &Node
            the * therefore on *self.node() produces a PLACE EXPRESSION
                ie. a place expression represents a memory location

            the consequences of having a place expression depend on how it is used
            the use case of interest here is if it used BY VALUE

            using a place expression by value means the compiler must either move or copy it out of that place
            because the reference is borrowed, moving it would not compile, as this function doesn't own it

            because on pub struct Node, the Copy trait is derived, a bitwise duplicate of the node can be obtained, which is what this expression yields

            note that the borrow ends at the end of this statement - the function now has an owned copy, and this is the last use of the borrow
            therefore, borrowing ends after last reference
        */
        let node = *self.get_node(node_id);

        // get the part of the original source covered by this node
        // a non-panicking way to slice the string, returning None if the range is out of bounds, or either endpoint lands mid UTF-8 char
        let chars_captured_by_node = input_string
            .get(node.start as usize..node.end as usize)
            .unwrap_or("<invalid source range>");

        /*
            {:indent$} placeholder gets "" as its argument, where using the Display trait, it pads the empty string "" out with indent-many of the default padding chars - a space
            {:?} gets node.kind, where using the Debug trait, it formats escapes - like newlines - and non-printables appropriately
            {} gets node.start, using the Display trait, simply displays the start index
            {} gets node.end, using the Display trait, simply displays the end index
            {:?} gets text, where using the Debug trait, it formats text, as explained above
        */
        let _ = writeln!(
            output_array,
            "{:indent$}{:?} [{}..{}] {:?}", // rest of arguments get put into each placeholder, as explained above
            "",
            node.node_type,
            node.start,
            node.end,
            chars_captured_by_node,
            indent = node_depth * 2,
        );

        // pre-order depth-first traversal on tree
        // (ie. visit, then push to stack (or in this case, updating the traversal node is sufficient))
        let mut child = node.first_child; // initialise node to track traversal
        while child != NO_NODE {
            self.print_node(child, input_string, node_depth + 1, output_array); // recurse

            child = self.get_node(child).next_sibling;
        }
    }
}
