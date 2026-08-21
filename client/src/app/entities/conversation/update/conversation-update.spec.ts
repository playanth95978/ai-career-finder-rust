import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { IConversation } from '../conversation.model';
import { ConversationService } from '../service/conversation.service';

import { ConversationFormService } from './conversation-form.service';
import { ConversationUpdate } from './conversation-update';

describe('Conversation Management Update Component', () => {
  let comp: ConversationUpdate;
  let fixture: ComponentFixture<ConversationUpdate>;
  let activatedRoute: ActivatedRoute;
  let conversationFormService: ConversationFormService;
  let conversationService: ConversationService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(ConversationUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    conversationFormService = TestBed.inject(ConversationFormService);
    conversationService = TestBed.inject(ConversationService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should update editForm', () => {
      const conversation: IConversation = { id: 10029 };

      activatedRoute.data = of({ conversation });
      comp.ngOnInit();

      expect(comp.conversation).toEqual(conversation);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IConversation>();
      const conversation = { id: 5450 };
      vitest.spyOn(conversationFormService, 'getConversation').mockReturnValue(conversation);
      vitest.spyOn(conversationService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ conversation });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(conversation);
      saveSubject.complete();

      // THEN
      expect(conversationFormService.getConversation).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(conversationService.update).toHaveBeenCalledWith(expect.objectContaining(conversation));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IConversation>();
      const conversation = { id: 5450 };
      vitest.spyOn(conversationFormService, 'getConversation').mockReturnValue({ id: null });
      vitest.spyOn(conversationService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ conversation: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(conversation);
      saveSubject.complete();

      // THEN
      expect(conversationFormService.getConversation).toHaveBeenCalled();
      expect(conversationService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IConversation>();
      const conversation = { id: 5450 };
      vitest.spyOn(conversationService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ conversation });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(conversationService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });
});
