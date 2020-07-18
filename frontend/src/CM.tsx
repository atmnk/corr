import React from 'react';
import './App.css';

interface ICounterProps {
  value?: number;
  onIncrement?: any;
  onDecrement?: any;
  onIncrementAsync?: any;
}
const Counter: React.FC<ICounterProps> =
({
  value,
  onIncrement,
  onDecrement,
  onIncrementAsync
}): JSX.Element => {
  return (
    <div>
      <button onClick={onIncrementAsync} className="button">
        Increment after 1 second
      </button>
      {' '}
      <button onClick={onIncrement} className="button">
        + Increment
      </button>
      {' '}
      <button onClick={onDecrement} className="button">
        - Decrement
      </button>
      <hr />
      <div>
        Clicked: {value} times
      </div>
    </div>
  )
};

export default Counter
