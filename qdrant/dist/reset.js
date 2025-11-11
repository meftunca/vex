#!/usr/bin/env node
/**
 * Vex Qdrant RAG System - Reset Collections
 * Deletes all collections and clears the database
 */
import VexQdrantClient from './client.js';
import config from './config.js';
import readline from 'readline';
async function confirm(question) {
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    });
    return new Promise((resolve) => {
        rl.question(question, (answer) => {
            rl.close();
            resolve(answer.toLowerCase() === 'y' || answer.toLowerCase() === 'yes');
        });
    });
}
async function main() {
    console.log('⚠️  RESET VEX QDRANT RAG SYSTEM ⚠️\n');
    console.log('This will DELETE all collections and indexed data:');
    console.log('  - vex_docs');
    console.log('  - vex_code');
    console.log('  - vex_todos');
    console.log('  - vex_examples');
    console.log('  - vex_tests');
    console.log();
    const shouldReset = await confirm('Are you sure you want to continue? (y/N): ');
    if (!shouldReset) {
        console.log('Reset cancelled.');
        return;
    }
    console.log('\n🗑️  Deleting collections...\n');
    const client = new VexQdrantClient();
    const collections = [
        config.collections.docs,
        config.collections.code,
        config.collections.todos,
        config.collections.examples,
        'vex_tests'
    ];
    let deleted = 0;
    let notFound = 0;
    for (const collection of collections) {
        try {
            await client.deleteCollection(collection);
            console.log(`✅ Deleted: ${collection}`);
            deleted++;
        }
        catch (error) {
            console.log(`⏭️  Skipped: ${collection} (not found)`);
            notFound++;
        }
    }
    console.log(`\n✅ Reset complete!`);
    console.log(`   Deleted: ${deleted} collections`);
    console.log(`   Not found: ${notFound} collections`);
    console.log('\n💡 Run "npm run sync" to re-index the documentation');
}
main().catch((error) => {
    console.error('❌ Error:', error.message);
    process.exit(1);
});
//# sourceMappingURL=reset.js.map