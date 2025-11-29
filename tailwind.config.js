/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: "selector",
  theme: {
    extend: {
      colors: {
        // macOS 风格系统蓝
        blue: {
          400: '#409CFF',
          500: '#0A84FF',
          600: '#0060DF',
        },
        // 自定义灰色系列（对齐 macOS 深色 System Gray）
        gray: {
          50: '#fafafa',   // bg-primary
          100: '#f4f4f5',  // bg-tertiary
          200: '#e4e4e7',  // border
          300: '#d4d4d8',  // border-hover
          400: '#a1a1aa',  // text-tertiary
          500: '#71717a',  // text-secondary
          600: '#636366',  // text-secondary-dark / systemGray2
          700: '#48484A',  // bg-tertiary-dark / separators
          800: '#3A3A3C',  // bg-secondary-dark
          900: '#2C2C2E',  // header / modal bg
          950: '#1C1C1E',  // app main bg
        },
        // 状态颜色
        green: {
          500: '#10b981',
          100: '#d1fae5',
        },
        red: {
          500: '#ef4444',
          100: '#fee2e2',
        },
        amber: {
          500: '#f59e0b',
          100: '#fef3c7',
        },
      },
      boxShadow: {
        'sm': '0 1px 2px 0 rgb(0 0 0 / 0.05)',
        'md': '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
        'lg': '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
      },
      borderRadius: {
        'sm': '0.375rem',
        'md': '0.5rem',
        'lg': '0.75rem',
        'xl': '0.875rem',
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', '"Helvetica Neue"', 'Arial', 'sans-serif'],
        mono: ['ui-monospace', 'SFMono-Regular', '"SF Mono"', 'Consolas', '"Liberation Mono"', 'Menlo', 'monospace'],
      },
      animation: {
        'fade-in': 'fadeIn 0.5s ease-out',
        'slide-up': 'slideUp 0.5s ease-out',
        'slide-down': 'slideDown 0.3s ease-out',
        'slide-in-right': 'slideInRight 0.3s ease-out',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(20px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        slideDown: {
          '0%': { transform: 'translateY(-100%)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        slideInRight: {
          '0%': { transform: 'translateX(100%)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        }
      }
    },
  },
  plugins: [],
}
