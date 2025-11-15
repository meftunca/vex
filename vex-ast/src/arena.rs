use typed_arena::Arena;

/// AST Arena for efficient memory management and reduced cloning
/// All AST nodes are allocated in arenas to avoid heap allocations and clones
pub struct AstArena<'ast> {
    pub programs: Arena<Program<'ast>>,
    pub imports: Arena<Import<'ast>>,
    pub items: Arena<Item<'ast>>,
    pub functions: Arena<Function<'ast>>,
    pub structs: Arena<Struct<'ast>>,
    pub traits: Arena<Trait<'ast>>,
    pub expressions: Arena<Expression<'ast>>,
    pub statements: Arena<Statement<'ast>>,
    pub blocks: Arena<Block<'ast>>,
    pub types: Arena<Type<'ast>>,
}

impl<'ast> AstArena<'ast> {
    pub fn new() -> Self {
        Self {
            programs: Arena::new(),
            imports: Arena::new(),
            items: Arena::new(),
            functions: Arena::new(),
            structs: Arena::new(),
            traits: Arena::new(),
            expressions: Arena::new(),
            statements: Arena::new(),
            blocks: Arena::new(),
            types: Arena::new(),
        }
    }

    pub fn alloc_program(&'ast self, program: Program<'ast>) -> &'ast Program<'ast> {
        self.programs.alloc(program)
    }

    pub fn alloc_import(&'ast self, import: Import<'ast>) -> &'ast Import<'ast> {
        self.imports.alloc(import)
    }

    pub fn alloc_item(&'ast self, item: Item<'ast>) -> &'ast Item<'ast> {
        self.items.alloc(item)
    }

    pub fn alloc_function(&'ast self, func: Function<'ast>) -> &'ast Function<'ast> {
        self.functions.alloc(func)
    }

    pub fn alloc_struct(&'ast self, strukt: Struct<'ast>) -> &'ast Struct<'ast> {
        self.structs.alloc(strukt)
    }

    pub fn alloc_trait(&'ast self, trait_: Trait<'ast>) -> &'ast Trait<'ast> {
        self.traits.alloc(trait_)
    }

    pub fn alloc_expression(&'ast self, expr: Expression<'ast>) -> &'ast Expression<'ast> {
        self.expressions.alloc(expr)
    }

    pub fn alloc_statement(&'ast self, stmt: Statement<'ast>) -> &'ast Statement<'ast> {
        self.statements.alloc(stmt)
    }

    pub fn alloc_block(&'ast self, block: Block<'ast>) -> &'ast Block<'ast> {
        self.blocks.alloc(block)
    }

    pub fn alloc_type(&'ast self, ty: Type<'ast>) -> &'ast Type<'ast> {
        self.types.alloc(ty)
    }
}