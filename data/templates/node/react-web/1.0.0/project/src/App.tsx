import {useState} from 'react';
import {create} from 'zustand';
import {z} from 'zod';
const title=z.string().trim().min(1).max(120);
const useTasks=create<{tasks:string[];add:(value:string)=>void}>(set=>({tasks:[],add:value=>set(state=>({tasks:[...state.tasks,title.parse(value)]}))}));
export function App(){const {tasks,add}=useTasks();const[value,setValue]=useState('');return <main><p className="eyebrow">YOUR WORKSPACE</p><h1>Make something useful.</h1><p>A local React app with the tools to keep building offline.</p><form onSubmit={event=>{event.preventDefault();if(title.safeParse(value).success){add(value);setValue('')}}}><label htmlFor="task">Next task</label><div className="entry"><input id="task" value={value} onChange={event=>setValue(event.target.value)} placeholder="What will you build?"/><button>Add task</button></div></form><ul>{tasks.map((task,index)=><li key={index}>{task}</li>)}</ul><footer>React · TypeScript · Vite · Local dependencies</footer></main>}
