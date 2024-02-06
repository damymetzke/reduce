/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.html"],
  theme: {
    extend: {
      colors: {
        button: {
          background: {
            alternate: "#bdc3c7",
            normal: "#eff0f1",
          },
          decoration: {
            focus: "#3daee9",
            hover: "#93cee9",
          },
          foreground: {
            active: "#3daee9",
            inactive: "#7f8c8d",
            link: "#2980b9",
            negative: "#da4453",
            neutral: "#f67400",
            normal: "#232629",
            positive: "#27ae60",
            visited: "#7f8c8d",
          },
        },
        view: {
          background: {
            alternate: "#eff0f1",
            normal: "#fcfcfc",
          },
          decoration: {
            focus: "#3daee9",
            hover: "#93cee9",
          },
          foreground: {
            active: "#3daee9",
            inactive: "#7f8c8d",
            link: "#2980b9",
            negative: "#da4453",
            neutral: "#f67400",
            normal: "#232629",
            positive: "#27ae60",
            visited: "#7f8c8d",
          },
        },
      }
    },
  },
  plugins: [],
}
