import 'dotenv/config';
import {mkdirSync} from 'node:fs';import {dirname} from 'node:path';
import {buildApp} from './app.mjs';
const database=process.env.DATABASE_PATH||'data/tasks.sqlite';mkdirSync(dirname(database),{recursive:true});
const app=await buildApp({database,logger:true});
await app.listen({host:process.env.HOST||'127.0.0.1',port:Number(process.env.PORT||3000)});
for(const signal of ['SIGINT','SIGTERM'])process.once(signal,async()=>{await app.close()});
