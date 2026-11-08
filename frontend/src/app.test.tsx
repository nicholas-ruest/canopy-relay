// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from './App';

describe('Canopy Relay shell', () => {
  it('presents six domain views and an explicit disconnected state', () => {
    render(<App />);
    expect(screen.getByText('Canopy Relay')).toBeInTheDocument();
    expect(screen.getByRole('navigation', { name: 'Canopy Relay views' })).toBeInTheDocument();
    for (const name of ['Habitat Map', 'Sensor Trust', 'Anomaly Workbench', 'Intervention Planner', 'Offline Mesh', 'Audit Vault']) {
      expect(screen.getByRole('button', { name: new RegExp(name) })).toBeInTheDocument();
    }
    expect(screen.getByText('Bring the living system into view.')).toBeInTheDocument();
    expect(screen.getByLabelText('Session token')).toHaveAttribute('type', 'password');
  });
});
