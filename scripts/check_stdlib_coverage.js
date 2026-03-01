const fs = require('fs');
const path = require('path');

const STDLIB_TYPES_PATH = path.join(__dirname, '../src/lsp/stdlib_types.json');
const BUILTINS_DIR = path.join(__dirname, '../src/runtime/builtins');

// Map of file/directory names to expected builtin names in stdlib_types.json
const MAPPING = {
    'array.rs': 'array', // Array functions might be under a different name or just 'array'
    'collections': 'array', // Assuming collections usually means array/map/set
    'encoding.rs': 'encoding',
    'ffi.rs': 'ffi',
    'io': 'fs', // io maps to fs builtin
    'json.rs': 'json',
    'math': 'math',
    'object.rs': 'object',
    'random.rs': 'random',
    'string': 'string', // string builtin
    'term.rs': 'term',
    'test.rs': 'test',
    'time.rs': 'time',
    'traits.rs': null, // traits.rs defines traits, likely covered by individual trait names
    'web': 'web',
    'mod.rs': null
};

function main() {
    console.log('Checking stdlib coverage...');

    if (!fs.existsSync(STDLIB_TYPES_PATH)) {
        console.error(`Error: ${STDLIB_TYPES_PATH} not found.`);
        process.exit(1);
    }

    const stdlibTypes = JSON.parse(fs.readFileSync(STDLIB_TYPES_PATH, 'utf8'));
    const documentedBuiltins = new Set(Object.keys(stdlibTypes.builtins || {}));

    console.log(`Found ${documentedBuiltins.size} documented builtins.`);
    
    const builtinFiles = fs.readdirSync(BUILTINS_DIR);
    let missingDocs = [];

    for (const file of builtinFiles) {
        if (file in MAPPING) {
            const expectedName = MAPPING[file];
            if (expectedName === null) continue;

            if (!documentedBuiltins.has(expectedName)) {
                // Check if maybe capitalization differs (e.g. Array vs array)
                // stdlib_types seem to use lowercase for modules (term, math) and PascalCase for types (BitXor)
                // But builtins are usually modules.
                
                // Special check for 'array' which might be 'Array' or similar
                // But stdlib_types has 'str', 'num' etc types? 
                // Let's assume builtins are the keys.
                
                missingDocs.push({ file, expectedName });
            }
        } else {
            console.warn(`Warning: unmapped file ${file} in builtins directory.`);
        }
    }

    if (missingDocs.length > 0) {
        console.error('\nMissing documentation for the following builtins:');
        for (const { file, expectedName } of missingDocs) {
            console.error(`- ${file} -> expected '${expectedName}'`);
        }
        process.exit(1);
    } else {
        console.log('\nAll mapped builtins are documented!');
    }
}

main();
