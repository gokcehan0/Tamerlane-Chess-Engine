import React, { createContext, useContext, useState, useEffect } from 'react';

// --- Types ---

export type AppTheme = 'dark' | 'light';

export interface BoardTheme {
    id: string;
    name: string;
    lightSquare: string;
    darkSquare: string;
    borderColor: string;
}

export interface ThemeColors {
    bg: string;          // Main background
    sidebar: string;     // Sidebar/Navbar background
    text: string;        // Primary text
    textSec: string;     // Secondary text
    border: string;      // Border colors
    card: string;        // Card/Modal background
    accent: string;      // Primary action color (blue)
    secondary: string;   // Secondary action color (for gradients)
    danger: string;      // Destructive action color (red)
    success: string;     // Success color (green)
    shadow: string;      // Box shadow
}

// --- Presets ---

export const BOARD_THEMES: BoardTheme[] = [
    { id: 'classic', name: 'Classic Green', lightSquare: '#eeeed2', darkSquare: '#769656', borderColor: '#5c7046' },
    { id: 'slate', name: 'Premium Slate (Original)', lightSquare: '#cbd5e1', darkSquare: '#475569', borderColor: '#334155' },
    { id: 'wood', name: 'Wooden', lightSquare: '#f0d9b5', darkSquare: '#b58863', borderColor: '#8b694a' },
    { id: 'icy', name: 'Icy Sea', lightSquare: '#dee3e6', darkSquare: '#8ca2ad', borderColor: '#697d87' },
    { id: 'contrast', name: 'High Contrast', lightSquare: '#ffffff', darkSquare: '#555555', borderColor: '#333333' },
    { id: 'tamerlane', name: 'Original Tamerlane', lightSquare: '#f0d9b5', darkSquare: '#b58863', borderColor: '#8b694a' } // Default for Tamerlane
];

const COLORS_DARK: ThemeColors = {
    bg: '#121212',
    sidebar: '#1e1e1e',
    text: '#e0e0e0',
    textSec: '#aaaaaa',
    border: '#333333',
    card: '#252525',
    accent: '#3b82f6',
    secondary: '#2563eb', // Darker blue
    danger: '#ef4444',
    success: '#22c55e',
    shadow: '0 4px 6px -1px rgba(0, 0, 0, 0.5), 0 2px 4px -1px rgba(0, 0, 0, 0.3)'
};

const COLORS_LIGHT: ThemeColors = {
    bg: '#f3f4f6',
    sidebar: '#ffffff',
    text: '#1f2937',
    textSec: '#374151',
    border: '#e5e7eb',
    card: '#ffffff',
    accent: '#2563eb',
    secondary: '#1d4ed8', // Darker blue
    danger: '#dc2626',
    success: '#16a34a',
    shadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)'
};

// --- Context ---

interface ThemeContextType {
    appTheme: AppTheme;
    setAppTheme: (theme: AppTheme) => void;
    boardTheme: BoardTheme;
    setBoardTheme: (id: string) => void;
    colors: ThemeColors;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function useTheme() {
    const context = useContext(ThemeContext);
    if (!context) throw new Error("useTheme must be used within a ThemeProvider");
    return context;
}

// Basic Cookie Helpers
function setCookie(name: string, value: string, days: number) {
    const d = new Date();
    d.setTime(d.getTime() + (days * 24 * 60 * 60 * 1000));
    document.cookie = `${name}=${value};expires=${d.toUTCString()};path=/;SameSite=Lax`;
}

function getCookie(name: string) {
    const match = document.cookie.match(new RegExp('(^| )' + name + '=([^;]+)'));
    return match ? match[2] : null;
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
    // Load from Cookie -> LocalStorage -> Default (Slate/Dark)
    const [appTheme, setAppThemeState] = useState<AppTheme>(() => {
        return (getCookie('appTheme') as AppTheme) ||
            (localStorage.getItem('appTheme') as AppTheme) ||
            'dark';
    });

    const [boardTheme, setBoardThemeState] = useState<BoardTheme>(() => {
        const savedId = getCookie('boardThemeId') || localStorage.getItem('boardThemeId');
        // Default to 'slate' (Our Original Tamerlane vibe)
        return BOARD_THEMES.find(t => t.id === savedId) ||
            BOARD_THEMES.find(t => t.id === 'slate') ||
            BOARD_THEMES[0];
    });

    // Sync to Storage AND Cookies (Robust Persistence)
    useEffect(() => {
        localStorage.setItem('appTheme', appTheme);
        setCookie('appTheme', appTheme, 365); // 1 Year
    }, [appTheme]);

    useEffect(() => {
        localStorage.setItem('boardThemeId', boardTheme.id);
        setCookie('boardThemeId', boardTheme.id, 365);
    }, [boardTheme]);

    const setAppTheme = (theme: AppTheme) => {
        setAppThemeState(theme);
    };

    const setBoardTheme = (id: string) => {
        const selected = BOARD_THEMES.find(t => t.id === id);
        if (selected) setBoardThemeState(selected);
    };

    const colors = appTheme === 'dark' ? COLORS_DARK : COLORS_LIGHT;

    const value = {
        appTheme,
        setAppTheme,
        boardTheme,
        setBoardTheme,
        colors
    };

    // Global Body Background Injection & Class Toggle
    useEffect(() => {
        document.body.style.backgroundColor = colors.bg;
        document.body.style.color = colors.text;

        // Toggle CSS class for CSS variable overrides
        if (appTheme === 'light') {
            document.body.classList.add('light-theme');
        } else {
            document.body.classList.remove('light-theme');
        }
    }, [colors, appTheme]);

    return (
        <ThemeContext.Provider value={value}>
            {children}
        </ThemeContext.Provider>
    );
}
