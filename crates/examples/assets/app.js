// Main application JavaScript
class App {
    constructor() {
        this.initialized = false;
        this.version = '1.0.0';
    }

    init() {
        console.log('Initializing Multi-Provider Asset Example App v' + this.version);
        this.setupEventListeners();
        this.loadEmbeddedData();
        this.initialized = true;
        console.log('App initialized successfully');
    }

    setupEventListeners() {
        document.addEventListener('DOMContentLoaded', () => {
            const buttons = document.querySelectorAll('.button');
            buttons.forEach(button => {
                button.addEventListener('click', (e) => {
                    console.log('Button clicked:', e.target.textContent);
                    this.handleButtonClick(e.target);
                });
            });
        });
    }

    handleButtonClick(button) {
        button.style.transform = 'scale(0.95)';
        setTimeout(() => {
            button.style.transform = 'scale(1)';
        }, 150);
    }

    loadEmbeddedData() {
        // This would normally load embedded configuration
        console.log('Loading embedded configuration...');
        return {
            theme: 'default',
            features: ['multi-provider', 'asset-generation', 'hot-reload'],
            debug: true
        };
    }

    getStats() {
        return {
            initialized: this.initialized,
            version: this.version,
            uptime: Date.now() - this.startTime
        };
    }
}

// Initialize app
const app = new App();
if (typeof window !== 'undefined') {
    app.startTime = Date.now();
    app.init();
}

// Export for testing
if (typeof module !== 'undefined' && module.exports) {
    module.exports = App;
}