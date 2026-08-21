import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import ConversationResolve from './route/conversation-routing-resolve.service';

const conversationRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/conversation').then(m => m.Conversation),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/conversation-detail').then(m => m.ConversationDetail),
    resolve: {
      conversation: ConversationResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/conversation-update').then(m => m.ConversationUpdate),
    resolve: {
      conversation: ConversationResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/conversation-update').then(m => m.ConversationUpdate),
    resolve: {
      conversation: ConversationResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default conversationRoute;
