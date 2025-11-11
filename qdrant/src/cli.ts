#!/usr/bin/env node
/**
 * Vex Qdrant RAG System - CLI Indexer
 */

import { Command } from "commander";
import DocumentationIndexer from "./indexer.js";

const program = new Command();

program
  .name("vex-indexer")
  .description("Index Vex documentation and codebase into Qdrant")
  .version("1.0.0");

program
  .command("all")
  .description("Index everything (specs, docs, code, examples, todos)")
  .action(async () => {
    const indexer = new DocumentationIndexer();
    await indexer.indexAll();
  });

program
  .command("specs")
  .description("Index only Specifications")
  .action(async () => {
    const indexer = new DocumentationIndexer();
    await indexer.indexSpecifications();
    await indexer.showStats();
  });

program
  .command("docs")
  .description("Index only documentation files")
  .action(async () => {
    const indexer = new DocumentationIndexer();
    await indexer.indexDocs();
    await indexer.showStats();
  });

program
  .command("code")
  .description("Index only code files")
  .action(async () => {
    const indexer = new DocumentationIndexer();
    await indexer.indexCode();
    await indexer.showStats();
  });

program
  .command("examples")
  .description("Index only example files")
  .action(async () => {
    const indexer = new DocumentationIndexer();
    await indexer.indexExamples();
    await indexer.showStats();
  });

program
  .command("todos")
  .description("Index only TODO items")
  .action(async () => {
    const indexer = new DocumentationIndexer();
    await indexer.indexTodos();
    await indexer.showStats();
  });

program
  .command("stats")
  .description("Show collection statistics")
  .action(async () => {
    const indexer = new DocumentationIndexer();
    await indexer.showStats();
  });

program.parse();
