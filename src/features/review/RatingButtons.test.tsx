import { render, screen } from '@testing-library/react';
import { RatingButtons } from './RatingButtons';
import { revealedFixture } from '../../test/reviewFixtures';
it('announces the interval without the extra estimated calendar-date description', () => {
  render(<RatingButtons frame={revealedFixture.payload} onRate={() => {}} disabled={false} />);
  const button = screen.getByRole('button', { name: /افتكرتها بسهولة.*بعد نحو ٣٠ يوم/ });
  expect(button).toHaveAccessibleDescription('');
  expect(button).not.toHaveAttribute('aria-describedby');
  expect(screen.queryByText(/الموعد المتوقع:/)).not.toBeInTheDocument();
  expect(screen.queryByText(/قد يتغير قليلًا عند تأخير التقييم/)).not.toBeInTheDocument();
});
