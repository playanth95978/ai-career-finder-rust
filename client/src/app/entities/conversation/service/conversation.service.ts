import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IConversation, NewConversation } from '../conversation.model';

export type PartialUpdateConversation = Partial<IConversation> & Pick<IConversation, 'id'>;

type RestOf<T extends IConversation | NewConversation> = Omit<T, 'createdAt' | 'lastMessageAt'> & {
  createdAt?: string | null;
  lastMessageAt?: string | null;
};

export type RestConversation = RestOf<IConversation>;

export type NewRestConversation = RestOf<NewConversation>;

export type PartialUpdateRestConversation = RestOf<PartialUpdateConversation>;

@Injectable()
export class ConversationsService {
  readonly conversationsParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly conversationsResource = httpResource<RestConversation[]>(() => {
    const params = this.conversationsParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of conversation that have been fetched. It is updated when the conversationsResource emits a new value.
   * In case of error while fetching the conversations, the signal is set to an empty array.
   */
  readonly conversations = computed(() =>
    (this.conversationsResource.hasValue() ? this.conversationsResource.value() : []).map(item => this.convertValueFromServer(item)),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/conversations');

  protected convertValueFromServer(restConversation: RestConversation): IConversation {
    return {
      ...restConversation,
      createdAt: restConversation.createdAt ? dayjs(restConversation.createdAt) : undefined,
      lastMessageAt: restConversation.lastMessageAt ? dayjs(restConversation.lastMessageAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class ConversationService extends ConversationsService {
  protected readonly http = inject(HttpClient);

  create(conversation: NewConversation): Observable<IConversation> {
    const copy = this.convertValueFromClient(conversation);
    return this.http.post<RestConversation>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(conversation: IConversation): Observable<IConversation> {
    const copy = this.convertValueFromClient(conversation);
    return this.http
      .put<RestConversation>(`${this.resourceUrl}/${encodeURIComponent(this.getConversationIdentifier(conversation))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(conversation: PartialUpdateConversation): Observable<IConversation> {
    const copy = this.convertValueFromClient(conversation);
    return this.http
      .patch<RestConversation>(`${this.resourceUrl}/${encodeURIComponent(this.getConversationIdentifier(conversation))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<IConversation> {
    return this.http
      .get<RestConversation>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<IConversation[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestConversation[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getConversationIdentifier(conversation: Pick<IConversation, 'id'>): number {
    return conversation.id;
  }

  compareConversation(o1: Pick<IConversation, 'id'> | null, o2: Pick<IConversation, 'id'> | null): boolean {
    return o1 && o2 ? this.getConversationIdentifier(o1) === this.getConversationIdentifier(o2) : o1 === o2;
  }

  addConversationToCollectionIfMissing<Type extends Pick<IConversation, 'id'>>(
    conversationCollection: Type[],
    ...conversationsToCheck: (Type | null | undefined)[]
  ): Type[] {
    const conversations: Type[] = conversationsToCheck.filter(isPresent);
    if (conversations.length > 0) {
      const conversationCollectionIdentifiers = conversationCollection.map(conversationItem =>
        this.getConversationIdentifier(conversationItem),
      );
      const conversationsToAdd = conversations.filter(conversationItem => {
        const conversationIdentifier = this.getConversationIdentifier(conversationItem);
        if (conversationCollectionIdentifiers.includes(conversationIdentifier)) {
          return false;
        }
        conversationCollectionIdentifiers.push(conversationIdentifier);
        return true;
      });
      return [...conversationsToAdd, ...conversationCollection];
    }
    return conversationCollection;
  }

  protected convertValueFromClient<T extends IConversation | NewConversation | PartialUpdateConversation>(conversation: T): RestOf<T> {
    return {
      ...conversation,
      createdAt: conversation.createdAt?.toJSON() ?? null,
      lastMessageAt: conversation.lastMessageAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestConversation): IConversation {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestConversation[]): IConversation[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
