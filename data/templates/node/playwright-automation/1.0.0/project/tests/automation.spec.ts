import { test, expect } from '@playwright/test';
import { createServer, type Server } from 'node:http';
let server: Server;
let baseURL: string;
test.beforeAll(async () => {
  server=createServer((request,response)=>{
    if(request.url==='/api/health'){response.setHeader('content-type','application/json');response.end(JSON.stringify({status:'ready',source:'local'}));return;}
    response.setHeader('content-type','text/html');
    response.end(`<!doctype html><html lang="en"><title>Offline automation</title><main><h1>Offline automation</h1><label>Task <input id="task"></label><button id="add">Add task</button><ul id="tasks"></ul></main><script>document.querySelector('#add').onclick=()=>{const row=document.createElement('li');row.textContent=document.querySelector('#task').value;document.querySelector('#tasks').append(row)};</script></html>`);
  });
  await new Promise<void>(resolve=>server.listen(0,'127.0.0.1',resolve));
  const address=server.address(); if(!address || typeof address==='string') throw Error('Expected TCP address');
  baseURL=`http://127.0.0.1:${address.port}`;
});
test.afterAll(async()=>{await new Promise<void>((resolve,reject)=>server.close(error=>error?reject(error):resolve()));});
test('browser form, accessible locators, and local screenshot',async({page},info)=>{
  await page.goto(baseURL);
  await expect(page.getByRole('heading',{name:'Offline automation'})).toBeVisible();
  await page.getByLabel('Task').fill('Build while disconnected');
  await page.getByRole('button',{name:'Add task'}).click();
  await expect(page.getByRole('listitem')).toHaveText('Build while disconnected');
  await page.screenshot({path:info.outputPath('offline-form.png'),fullPage:true});
});
test('API request and JSON contract',async({request})=>{
  const response=await request.get(`${baseURL}/api/health`);expect(response.ok()).toBeTruthy();
  expect(await response.json()).toEqual({status:'ready',source:'local'});
});
