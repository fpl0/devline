import { expect, test } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import { App } from './App';

test('adds a validated task using the local form', () => {
  render(<App />);
  const input = screen.getByLabelText('Next task') as HTMLInputElement;

  fireEvent.change(input, { target: { value: 'Build offline' } });
  fireEvent.click(screen.getByRole('button', { name: 'Add task' }));

  expect(screen.getByRole('listitem').textContent).toBe('Build offline');
  expect(input.value).toBe('');
});
