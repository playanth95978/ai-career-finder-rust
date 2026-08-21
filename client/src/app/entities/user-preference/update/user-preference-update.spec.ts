import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { UserPreferenceService } from '../service/user-preference.service';
import { IUserPreference } from '../user-preference.model';

import { UserPreferenceFormService } from './user-preference-form.service';
import { UserPreferenceUpdate } from './user-preference-update';

describe('UserPreference Management Update Component', () => {
  let comp: UserPreferenceUpdate;
  let fixture: ComponentFixture<UserPreferenceUpdate>;
  let activatedRoute: ActivatedRoute;
  let userPreferenceFormService: UserPreferenceFormService;
  let userPreferenceService: UserPreferenceService;

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

    fixture = TestBed.createComponent(UserPreferenceUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    userPreferenceFormService = TestBed.inject(UserPreferenceFormService);
    userPreferenceService = TestBed.inject(UserPreferenceService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should update editForm', () => {
      const userPreference: IUserPreference = { id: 7916 };

      activatedRoute.data = of({ userPreference });
      comp.ngOnInit();

      expect(comp.userPreference).toEqual(userPreference);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IUserPreference>();
      const userPreference = { id: 31342 };
      vitest.spyOn(userPreferenceFormService, 'getUserPreference').mockReturnValue(userPreference);
      vitest.spyOn(userPreferenceService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ userPreference });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(userPreference);
      saveSubject.complete();

      // THEN
      expect(userPreferenceFormService.getUserPreference).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(userPreferenceService.update).toHaveBeenCalledWith(expect.objectContaining(userPreference));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IUserPreference>();
      const userPreference = { id: 31342 };
      vitest.spyOn(userPreferenceFormService, 'getUserPreference').mockReturnValue({ id: null });
      vitest.spyOn(userPreferenceService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ userPreference: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(userPreference);
      saveSubject.complete();

      // THEN
      expect(userPreferenceFormService.getUserPreference).toHaveBeenCalled();
      expect(userPreferenceService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IUserPreference>();
      const userPreference = { id: 31342 };
      vitest.spyOn(userPreferenceService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ userPreference });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(userPreferenceService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });
});
