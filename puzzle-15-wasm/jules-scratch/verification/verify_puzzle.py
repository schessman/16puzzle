from playwright.sync_api import sync_playwright, expect

def run_verification():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()

        try:
            # Navigate to the page. The server is running at the project root.
            page.goto("http://localhost:8000/index.html", timeout=60000)

            # Wait for the canvas and controls to be visible
            canvas = page.locator("#puzzle_canvas")
            expect(canvas).to_be_visible()

            restart_button = page.get_by_role("button", name="Restart")
            expect(restart_button).to_be_visible()

            move_counter = page.locator("#move-count")
            expect(move_counter).to_have_text("0")

            # Take a screenshot to verify the initial state
            page.screenshot(path="jules-scratch/verification/verification.png")

            print("Verification successful, screenshot saved.")

        except Exception as e:
            print(f"An error occurred during verification: {e}")
            # Take a screenshot on failure for debugging
            page.screenshot(path="jules-scratch/verification/verification_error.png")

        finally:
            browser.close()

if __name__ == "__main__":
    run_verification()