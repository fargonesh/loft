
document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('doc-search');
    const sidebar = document.querySelector('.sidebar');
    
    if (!searchInput || !sidebar) return;

    // Create results container
    const resultsContainer = document.createElement('div');
    resultsContainer.id = 'search-results';
    resultsContainer.style.display = 'none';
    sidebar.insertBefore(resultsContainer, sidebar.children[2]); // Insert after title and search input

    searchInput.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase();
        
        if (query.length < 2) {
            resultsContainer.style.display = 'none';
            document.querySelectorAll('.nav-section').forEach(el => el.style.display = 'block');
            return;
        }

        // Hide normal nav
        document.querySelectorAll('.nav-section').forEach(el => el.style.display = 'none');
        resultsContainer.style.display = 'block';
        resultsContainer.innerHTML = '';

        const results = SEARCH_INDEX.filter(item => 
            item.name.toLowerCase().includes(query) || 
            (item.doc && item.doc.toLowerCase().includes(query))
        ).slice(0, 20);

        if (results.length === 0) {
            resultsContainer.innerHTML = '<div class="no-results">No results found</div>';
            return;
        }

        const ul = document.createElement('ul');
        results.forEach(item => {
            const li = document.createElement('li');
            const a = document.createElement('a');
            a.href = item.url;
            a.innerHTML = `<span class="result-name">${item.name}</span> <span class="result-type">${item.type}</span>`;
            li.appendChild(a);
            ul.appendChild(li);
        });
        resultsContainer.appendChild(ul);
    });
});
