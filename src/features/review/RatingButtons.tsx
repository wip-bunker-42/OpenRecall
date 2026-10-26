import { useId } from 'react';
import type { ReviewFrame } from '../../shared/generated/ReviewFrame';
import type { Rating } from '../../shared/generated/Rating';
import { interval, dateTime } from '../../shared/format';
export const ratingLabels: Record<Rating, string> = {
  again: 'نسيت',
  hard: 'افتكرتها بصعوبة',
  good: 'افتكرتها بشكل طبيعي',
  easy: 'افتكرتها بسهولة',
};
export const ratings: Rating[] = ['again', 'hard', 'good', 'easy'];
export function RatingButtons({
  frame,
  onRate,
  disabled,
  timezone,
}: {
  frame: ReviewFrame;
  onRate: (rating: Rating) => void;
  disabled: boolean;
  timezone: string;
}) {
  const id = useId();
  if (!frame.previews) return null;
  return (
    <fieldset className="rating-fieldset" disabled={disabled}>
      <legend>تقييم التذكر</legend>
      <div className="rating-grid">
        {ratings.map((rating, i) => {
          const approximate = interval(frame.previews![rating].intervalSeconds);
          const due = frame.previewDueAt?.[rating];
          return (
            <div key={rating}>
              <button
                onClick={() => onRate(rating)}
                className={`rating rating-${rating}`}
                aria-describedby={due ? `${id}-${rating}` : undefined}
              >
                <span>{ratingLabels[rating]}</span>
                <span className="rating-interval">{approximate}</span>
                <kbd aria-hidden="true">{i + 1}</kbd>
              </button>
              {due && (
                <span className="sr-only" id={`${id}-${rating}`}>
                  الموعد المتوقع: {dateTime(due, timezone)}. قد يتغير قليلًا عند تأخير التقييم.
                </span>
              )}
            </div>
          );
        })}
      </div>
    </fieldset>
  );
}
