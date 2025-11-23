import { render, screen } from '@testing-library/react';
import App from './App';

describe('Context web shell', () => {
  test('renders brand and mission copy', () => {
    render(<App />);

    expect(screen.getAllByText(/Context/).length).toBeGreaterThan(0);
    expect(screen.getByText(/agent-ready knowledge console/i)).toBeInTheDocument();
  });

  test('includes search workspace controls', () => {
    render(<App />);

    const searchBox = screen.getByRole('searchbox');
    expect(searchBox).toHaveAttribute('placeholder', expect.stringMatching(/search/i));

    expect(screen.getByLabelText(/project/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /new document/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /open cli log/i })).toBeInTheDocument();
  });

  test('shows keyboard help and a seeded command log', () => {
    render(<App />);

    expect(screen.getByText(/keyboard shortcuts/i)).toBeInTheDocument();
    expect(screen.getByText(/focus search/i)).toBeInTheDocument();

    const logEntries = screen.getAllByTestId('command-log-entry');
    expect(logEntries.length).toBeGreaterThanOrEqual(2);
  });
});
