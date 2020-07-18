import { createSlice, PayloadAction } from '@reduxjs/toolkit';

type CurrentDisplayState = {
  clicks: number
}

let initialState: CurrentDisplayState = {
  clicks: 0,
}

const countSlice = createSlice({
  name: 'count',
  initialState,
  reducers: {
    addCount(state, action: PayloadAction<number>) {
      state.clicks += action.payload
    },
    minusCount(state, action: PayloadAction<number>) {
      state.clicks -= action.payload
    }
  }
})

export const {
  addCount,
  minusCount
} = countSlice.actions

export default countSlice.reducer
