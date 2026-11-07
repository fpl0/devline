import Fastify from 'fastify';
import cors from '@fastify/cors';
import helmet from '@fastify/helmet';
import {DatabaseSync} from 'node:sqlite';
import {z} from 'zod';
export async function buildApp({database=':memory:',logger=false}={}) {
 const app=Fastify({logger}); const db=new DatabaseSync(database);
 db.exec('CREATE TABLE IF NOT EXISTS tasks(id INTEGER PRIMARY KEY,title TEXT NOT NULL)');
 await app.register(cors,{origin:false}); await app.register(helmet);
 const input=z.object({title:z.string().trim().min(1).max(120)});
 app.get('/health',async()=>({status:'ready',storage:'sqlite'}));
 app.get('/tasks',async()=>db.prepare('SELECT id,title FROM tasks ORDER BY id').all());
 app.post('/tasks',async(request,reply)=>{const result=input.safeParse(request.body);if(!result.success)return reply.code(400).send({error:'A task needs a title of 1–120 characters.'});const row=db.prepare('INSERT INTO tasks(title) VALUES(?)').run(result.data.title);return reply.code(201).send({id:Number(row.lastInsertRowid),title:result.data.title})});
 app.delete('/tasks/:id',async(request,reply)=>{const id=Number(request.params.id);if(!Number.isSafeInteger(id)||id<1)return reply.code(400).send({error:'Invalid task ID'});const changed=db.prepare('DELETE FROM tasks WHERE id=?').run(id).changes;return changed?reply.code(204).send():reply.code(404).send({error:'Task not found'})});
 app.addHook('onClose',async()=>db.close());return app;
}
